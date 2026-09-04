// SPDX-License-Identifier: Apache-2.0
//! Framework-level async executor for GUI tasks.
//!
//! Futures spawned here run entirely on the thread that owns the widget
//! tree, so they may freely capture `Rc`/`RefCell` state or hook state -
//! no `Send` bound is required, unlike `tokio::spawn`. A task's own
//! `Waker` is still safe to invoke from any thread, since some sources of
//! wakeup (a background HTTP client, a timer thread, ...) don't run on
//! the GUI thread themselves.

use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct TaskId(u64);

impl TaskId {
    // A global (not per-thread) counter, so an id can never collide with
    // one from another thread even though the tasks themselves are kept
    // in a thread-local table.
    fn next() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

type BoxedTask = Pin<Box<dyn Future<Output = ()>>>;

thread_local! {
    static TASKS: RefCell<HashMap<TaskId, BoxedTask>> = RefCell::new(HashMap::new());
}

// Ids waiting to be polled. Deliberately not a thread_local: a task's
// Waker can be called from any thread, and only the id needs to cross
// that boundary - the future itself never leaves the thread that owns it.
static READY: Mutex<Vec<TaskId>> = Mutex::new(Vec::new());

/// Lets the executor wake its host event loop from any thread. The
/// platform runtime (e.g. `xenframe`) implements this once over its own
/// event loop proxy and registers it via [`set_executor_waker`].
#[cfg(not(target_arch = "wasm32"))]
pub trait ExecutorWaker: Send + Sync {
    /// Returns or updates the `wake` value.
    fn wake(&self);
}

#[cfg(target_arch = "wasm32")]
/// Wakes the host event loop when a spawned task becomes ready.
pub trait ExecutorWaker {
    /// Requests that the host poll the GUI-thread executor.
    fn wake(&self);
}

#[cfg(not(target_arch = "wasm32"))]
static EXECUTOR_WAKER: Mutex<Option<Arc<dyn ExecutorWaker>>> = Mutex::new(None);

#[cfg(target_arch = "wasm32")]
thread_local! {
    static EXECUTOR_WAKER: RefCell<Option<Arc<dyn ExecutorWaker>>> = const { RefCell::new(None) };
}

#[cfg(not(target_arch = "wasm32"))]
/// Updates the `set_executor_waker` value.
/// Installs the callback used to wake the host event loop.
pub fn set_executor_waker(waker: Arc<dyn ExecutorWaker>) {
    *EXECUTOR_WAKER.lock().unwrap() = Some(waker);
}

#[cfg(target_arch = "wasm32")]
/// Installs the callback used to wake the host event loop.
pub fn set_executor_waker(waker: Arc<dyn ExecutorWaker>) {
    EXECUTOR_WAKER.with(|cell| {
        *cell.borrow_mut() = Some(waker);
    });
}

fn wake_task(id: TaskId) {
    READY.lock().unwrap().push(id);

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(waker) = EXECUTOR_WAKER.lock().unwrap().as_ref() {
        waker.wake();
    }

    #[cfg(target_arch = "wasm32")]
    EXECUTOR_WAKER.with(|cell| {
        if let Some(waker) = cell.borrow().as_ref() {
            waker.wake();
        }
    });
}

struct TaskWaker(TaskId);

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        wake_task(self.0);
    }

    fn wake_by_ref(self: &Arc<Self>) {
        wake_task(self.0);
    }
}

/// Spawns a future onto the GUI-thread executor.
///
/// The future's output is discarded, so fallible futures
/// (`Result<T, E>`) can be spawned directly without an extra `.map()`.
/// Must be called from the same thread that later drives the executor
/// via [`poll`] - in practice, the GUI thread.
pub fn spawn<F>(future: F)
where
    F: Future + 'static,
{
    let id = TaskId::next();
    let boxed: BoxedTask = Box::pin(async move {
        future.await;
    });
    TASKS.with(|tasks| tasks.borrow_mut().insert(id, boxed));
    wake_task(id);
}

/// Polls every task currently marked ready, dropping it once it
/// completes. Safe to call every frame regardless of whether anything
/// actually woke up - an empty ready queue returns immediately.
pub fn poll() {
    let ready: Vec<TaskId> = {
        let mut queue = READY.lock().unwrap();
        if queue.is_empty() {
            return;
        }
        std::mem::take(&mut *queue)
    };

    for id in ready {
        // Already completed (or spawned on a different thread) - a task
        // can be woken more than once before its next poll.
        let Some(mut future) = TASKS.with(|tasks| tasks.borrow_mut().remove(&id)) else {
            continue;
        };

        let waker = Waker::from(Arc::new(TaskWaker(id)));
        let mut cx = Context::from_waker(&waker);

        match future.as_mut().poll(&mut cx) {
            Poll::Ready(()) => {}
            Poll::Pending => {
                TASKS.with(|tasks| tasks.borrow_mut().insert(id, future));
            }
        }
    }
}

/// Drops every task still pending on this thread, without polling them
/// again. Called when the application exits.
pub fn cancel_all() {
    TASKS.with(|tasks| tasks.borrow_mut().clear());
}

// --- spawn_blocking support -------------------------------------------------
//
// Runs a blocking/synchronous closure (HID, filesystem, WMI, blocking
// network calls, ...) on its own std::thread instead of stalling the GUI
// thread. Unlike `spawn`, the closure and its result genuinely cross a
// thread boundary and must be `Send + 'static`; the returned future
// itself is never required to be `Send` since it's only ever polled from
// the GUI thread, same as every other task in this module.

#[cfg(not(target_arch = "wasm32"))]
struct BlockingShared<T> {
    result: Option<T>,
    waker: Option<Waker>,
}

#[cfg(not(target_arch = "wasm32"))]
/// Data and behavior represented by `SpawnBlocking`.
pub struct SpawnBlocking<T> {
    shared: Arc<Mutex<BlockingShared<T>>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl<T> Future for SpawnBlocking<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        // Checking the result and (if absent) registering the waker under
        // the same lock closes the window where the background thread
        // could finish between those two steps - the exact sequence that
        // would otherwise cause a lost wakeup.
        let mut guard = self.shared.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(result) = guard.result.take() {
            Poll::Ready(result)
        } else {
            guard.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

/// Runs `f` on a dedicated background `std::thread`, leaving the GUI
/// thread free to keep handling input/paint. Await the returned future
/// from within a `spawn`-ed task (or `use_resource`) to pick the result
/// back up on the GUI thread once it's ready.
///
/// Each call spawns its own thread - fine for occasional blocking calls;
/// reach for a real thread pool if this ever needs to run at high
/// frequency.
///
/// ```compile_fail
/// use std::rc::Rc;
/// use xengui::task::spawn_blocking;
///
/// let state = Rc::new(5);
/// let _ = spawn_blocking(move || {
///     // `Rc` is not `Send`, so this fails to compile - exactly the
///     // guardrail that keeps GUI state off the background thread.
///     *state
/// });
/// ```
///
/// # Panics
/// A panic inside `f` is caught on the background thread and simply never
/// resolves the future (it stays `Pending` forever) instead of unwinding
/// into the GUI executor. Under this workspace's `panic = "abort"`
/// release profile the process still aborts, matching plain
/// `std::thread::spawn`'s existing behavior; only unwind builds (e.g.
/// `cargo test`) benefit from the catch.
#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_blocking<F, T>(f: F) -> SpawnBlocking<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let shared = Arc::new(Mutex::new(BlockingShared {
        result: None,
        waker: None,
    }));
    let worker_shared = shared.clone();

    std::thread::spawn(move || {
        use std::panic;

        let outcome = panic::catch_unwind(panic::AssertUnwindSafe(f));

        let Ok(value) = outcome else {
            // Nothing here can produce a `T` for a panicked closure, and
            // this thread must not propagate the unwind onto the GUI
            // thread - the task simply never completes (see doc above).
            return;
        };

        // The future may already have been dropped (its `Arc` refcount
        // down to just this one) - storing the result is still safe, it's
        // simply never read back out, and `waker` will be `None`.
        let waker = {
            let mut guard = worker_shared.lock().unwrap_or_else(|p| p.into_inner());
            guard.result = Some(value);
            guard.waker.take()
        };

        if let Some(waker) = waker {
            waker.wake();
        }
    });

    SpawnBlocking { shared }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;
    #[cfg(not(target_arch = "wasm32"))]
    use std::time::Duration;

    // Serializes tests that touch the executor's process-wide ready
    // queue and waker slot, since the test harness runs tests in
    // parallel by default.
    fn test_guard() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[test]
    fn spawned_future_runs_to_completion() {
        let _guard = test_guard();
        let ran = Rc::new(Cell::new(false));
        let ran_clone = ran.clone();

        spawn(async move {
            ran_clone.set(true);
        });
        poll();

        assert!(ran.get());
        cancel_all();
    }

    #[test]
    fn pending_future_resumes_after_self_wake() {
        let _guard = test_guard();

        struct WakeOnceThenReady {
            polled: bool,
        }

        impl Future for WakeOnceThenReady {
            type Output = ();

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
                if self.polled {
                    Poll::Ready(())
                } else {
                    self.polled = true;
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
        }

        let steps = Rc::new(Cell::new(0));
        let steps_for_future = steps.clone();

        spawn(async move {
            steps_for_future.set(steps_for_future.get() + 1);
            (WakeOnceThenReady { polled: false }).await;
            steps_for_future.set(steps_for_future.get() + 1);
        });

        // First poll enters the async block and hits the inner future,
        // which re-wakes itself before returning Pending.
        poll();
        // That self-wake already landed back in the ready queue, so a
        // second drain finishes the task with no external stimulus.
        poll();

        assert_eq!(steps.get(), 2);
        cancel_all();
    }

    #[test]
    fn cancel_all_drops_pending_tasks() {
        let _guard = test_guard();

        struct MarkOnDrop(Rc<Cell<bool>>);

        impl Drop for MarkOnDrop {
            fn drop(&mut self) {
                self.0.set(true);
            }
        }

        struct NeverReady(#[allow(dead_code)] MarkOnDrop);

        impl Future for NeverReady {
            type Output = ();

            fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
                Poll::Pending
            }
        }

        let dropped = Rc::new(Cell::new(false));
        let marker = MarkOnDrop(dropped.clone());

        spawn(async move {
            NeverReady(marker).await;
        });
        poll();
        assert!(!dropped.get());

        cancel_all();
        assert!(dropped.get());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn spawn_blocking_runs_off_thread_and_returns_result() {
        let _guard = test_guard();
        let main_thread_id = std::thread::current().id();
        let done = Rc::new(Cell::new(None));
        let done_clone = done.clone();

        spawn(async move {
            let worker_id = spawn_blocking(|| std::thread::current().id()).await;
            done_clone.set(Some(worker_id));
        });

        // Drives the executor until the blocking task's own waker fires.
        for _ in 0..200 {
            poll();
            if done.get().is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        let worker_id = done.get().expect("spawn_blocking never completed");
        assert_ne!(worker_id, main_thread_id);
        cancel_all();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn spawn_blocking_future_pending_then_ready() {
        let _guard = test_guard();

        let mut cx = Context::from_waker(Waker::noop());

        let mut fut = spawn_blocking(|| {
            std::thread::sleep(Duration::from_millis(30));
            99
        });

        let first = Pin::new(&mut fut).poll(&mut cx);
        assert!(matches!(first, Poll::Pending));

        std::thread::sleep(Duration::from_millis(80));
        let second = Pin::new(&mut fut).poll(&mut cx);
        assert_eq!(second, Poll::Ready(99));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn spawn_blocking_panic_does_not_break_executor() {
        let _guard = test_guard();

        spawn(async move {
            // Never completes since the closure panics - documented
            // behavior, not a bug in this test.
            let _: i32 = spawn_blocking(|| panic!("boom")).await;
        });

        for _ in 0..50 {
            poll();
            std::thread::sleep(Duration::from_millis(5));
        }

        // The executor must still work normally for unrelated tasks.
        let ran = Rc::new(Cell::new(false));
        let ran_clone = ran.clone();
        spawn(async move {
            ran_clone.set(true);
        });
        poll();

        assert!(ran.get());
        cancel_all();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn spawn_blocking_dropped_before_completion_is_ignored() {
        let _guard = test_guard();
        let (tx, rx) = std::sync::mpsc::channel();

        {
            let fut = spawn_blocking(move || {
                std::thread::sleep(Duration::from_millis(20));
                let _ = tx.send(());
                7
            });
            drop(fut); // dropped well before the background thread finishes
        }

        // The background thread still runs to completion; its result is
        // simply never observed by anything since the future is gone.
        rx.recv_timeout(Duration::from_secs(1))
            .expect("worker thread never finished");
        cancel_all();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn spawn_blocking_many_concurrent_do_not_lose_wakeups() {
        let _guard = test_guard();
        const N: usize = 50;
        let completed = Rc::new(Cell::new(0usize));

        for i in 0..N {
            let completed = completed.clone();
            spawn(async move {
                let v = spawn_blocking(move || i).await;
                assert_eq!(v, i);
                completed.set(completed.get() + 1);
            });
        }

        for _ in 0..500 {
            poll();
            if completed.get() == N {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        assert_eq!(completed.get(), N);
        cancel_all();
    }
}
