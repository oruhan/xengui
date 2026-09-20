// SPDX-License-Identifier: Apache-2.0
//! Framework-level async executor for GUI tasks.
//!
//! Futures spawned here run entirely on the thread that owns the widget
//! tree, so they may freely capture `Rc`/`RefCell` state or hook state -
//! no `Send` bound is required, unlike `tokio::spawn`. A task's own
//! `Waker` is still safe to invoke from any thread, since some sources of
//! wakeup (a background HTTP client, a timer thread, ...) don't run on
//! the GUI thread themselves.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};

/// Yields a GUI task until the next executor poll. This is useful for work
/// that must happen only after the current committed tree has painted once.
pub async fn yield_now() {
    let mut yielded = false;
    std::future::poll_fn(move |cx| {
        if yielded {
            Poll::Ready(())
        } else {
            yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    })
    .await
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct TaskId(u64);

type BoxedTask = Pin<Box<dyn Future<Output = ()>>>;

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

struct Scheduler {
    ready: Mutex<Vec<TaskId>>,
    #[cfg(not(target_arch = "wasm32"))]
    host_waker: Mutex<Option<Arc<dyn ExecutorWaker>>>,
    #[cfg(target_arch = "wasm32")]
    runtime_id: u64,
}

impl Scheduler {
    fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        static NEXT_RUNTIME_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

        Self {
            ready: Mutex::new(Vec::new()),
            #[cfg(not(target_arch = "wasm32"))]
            host_waker: Mutex::new(None),
            #[cfg(target_arch = "wasm32")]
            runtime_id: NEXT_RUNTIME_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        }
    }

    fn enqueue(&self, id: TaskId) {
        self.ready.lock().unwrap().push(id);

        #[cfg(not(target_arch = "wasm32"))]
        let host_waker = self.host_waker.lock().unwrap().clone();
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(waker) = host_waker {
            waker.wake();
        }

        #[cfg(target_arch = "wasm32")]
        WASM_HOST_WAKERS.with(|wakers| {
            let host_waker = wakers.borrow().get(&self.runtime_id).cloned();
            if let Some(waker) = host_waker {
                waker.wake();
            }
        });
    }

    fn take_ready(&self) -> Vec<TaskId> {
        let mut ready = self.ready.lock().unwrap();
        std::mem::take(&mut *ready)
    }
}

struct TaskWaker {
    id: TaskId,
    scheduler: Arc<Scheduler>,
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.scheduler.enqueue(self.id);
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.scheduler.enqueue(self.id);
    }
}

struct RuntimeInner {
    owner: std::thread::ThreadId,
    next_id: Cell<u64>,
    tasks: RefCell<HashMap<TaskId, BoxedTask>>,
    scheduler: Arc<Scheduler>,
}

thread_local! {
    static CURRENT_RUNTIME: RefCell<Weak<RuntimeInner>> = const { RefCell::new(Weak::new()) };
    #[cfg(target_arch = "wasm32")]
    static WASM_HOST_WAKERS: RefCell<HashMap<u64, Arc<dyn ExecutorWaker>>> = RefCell::new(HashMap::new());
}

#[cfg(target_arch = "wasm32")]
impl Drop for RuntimeInner {
    fn drop(&mut self) {
        WASM_HOST_WAKERS.with(|wakers| {
            wakers.borrow_mut().remove(&self.scheduler.runtime_id);
        });
    }
}

/// An isolated GUI-task executor owned by one application runtime.
///
/// Futures never leave the creating thread. Only the scheduler queue carried
/// by their `Waker`s is thread-safe, so an external wake always returns to the
/// runtime that owns the future.
#[derive(Clone)]
pub struct Runtime {
    inner: Rc<RuntimeInner>,
}

impl Runtime {
    /// Creates an empty runtime owned by the current thread.
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RuntimeInner {
                owner: std::thread::current().id(),
                next_id: Cell::new(0),
                tasks: RefCell::new(HashMap::new()),
                scheduler: Arc::new(Scheduler::new()),
            }),
        }
    }

    fn assert_owner(&self) {
        assert_eq!(
            self.inner.owner,
            std::thread::current().id(),
            "GUI task runtime used from a non-owner thread"
        );
    }

    /// Makes this runtime the target of the module-level [`spawn`] helpers on
    /// its owner thread.
    pub fn activate(&self) {
        self.assert_owner();
        CURRENT_RUNTIME.with(|current| {
            *current.borrow_mut() = Rc::downgrade(&self.inner);
        });
    }

    /// Installs the callback used to wake this runtime's host event loop.
    pub fn set_executor_waker(&self, waker: Arc<dyn ExecutorWaker>) {
        self.assert_owner();
        #[cfg(not(target_arch = "wasm32"))]
        {
            *self.inner.scheduler.host_waker.lock().unwrap() = Some(waker);
        }
        #[cfg(target_arch = "wasm32")]
        {
            WASM_HOST_WAKERS.with(|wakers| {
                wakers
                    .borrow_mut()
                    .insert(self.inner.scheduler.runtime_id, waker);
            });
        }
    }

    /// Spawns a future on this runtime.
    pub fn spawn<F>(&self, future: F)
    where
        F: Future + 'static,
    {
        self.activate();
        let next = self.inner.next_id.get();
        self.inner
            .next_id
            .set(next.checked_add(1).expect("GUI task id space exhausted"));
        let id = TaskId(next);
        let boxed: BoxedTask = Box::pin(async move {
            future.await;
        });
        self.inner.tasks.borrow_mut().insert(id, boxed);
        self.inner.scheduler.enqueue(id);
    }

    /// Polls every task currently ready for this runtime.
    pub fn poll(&self) {
        self.activate();
        let ready = self.inner.scheduler.take_ready();
        if ready.is_empty() {
            return;
        }

        for id in ready {
            let Some(mut future) = self.inner.tasks.borrow_mut().remove(&id) else {
                continue;
            };
            let waker = Waker::from(Arc::new(TaskWaker {
                id,
                scheduler: self.inner.scheduler.clone(),
            }));
            let mut cx = Context::from_waker(&waker);
            if future.as_mut().poll(&mut cx).is_pending() {
                self.inner.tasks.borrow_mut().insert(id, future);
            }
        }
    }

    /// Drops every task still pending in this runtime.
    pub fn cancel_all(&self) {
        self.assert_owner();
        self.inner.tasks.borrow_mut().clear();
        let _ = self.inner.scheduler.take_ready();
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

fn current_runtime() -> Runtime {
    CURRENT_RUNTIME.with(|current| Runtime {
        inner: current
            .borrow()
            .upgrade()
            .expect("no active GUI task runtime on this thread"),
    })
}

/// Installs the callback used to wake the active host event loop.
pub fn set_executor_waker(waker: Arc<dyn ExecutorWaker>) {
    current_runtime().set_executor_waker(waker);
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
    current_runtime().spawn(future);
}

/// Polls every task currently marked ready, dropping it once it
/// completes. Safe to call every frame regardless of whether anything
/// actually woke up - an empty ready queue returns immediately.
pub fn poll() {
    current_runtime().poll();
}

/// Drops every task still pending on this thread, without polling them
/// again. Called when the application exits.
pub fn cancel_all() {
    current_runtime().cancel_all();
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

    struct TestGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
        _runtime: Runtime,
    }

    // The lock keeps timing-sensitive spawn_blocking tests deterministic;
    // every test still receives a distinct executor runtime.
    fn test_guard() -> TestGuard {
        static LOCK: Mutex<()> = Mutex::new(());
        let lock = LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let runtime = Runtime::new();
        runtime.activate();
        TestGuard {
            _lock: lock,
            _runtime: runtime,
        }
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
    fn runtimes_on_the_same_thread_keep_tasks_and_ready_queues_isolated() {
        let _guard = test_guard();
        let first_ran = Rc::new(Cell::new(false));
        let second_ran = Rc::new(Cell::new(false));
        let first = Runtime::new();
        let second = Runtime::new();

        let marker = first_ran.clone();
        first.spawn(async move { marker.set(true) });
        let marker = second_ran.clone();
        second.spawn(async move { marker.set(true) });

        first.poll();
        assert!(first_ran.get());
        assert!(!second_ran.get());

        second.poll();
        assert!(second_ran.get());
    }

    #[test]
    fn task_spawned_while_polling_stays_on_the_polling_runtime() {
        let _guard = test_guard();
        let runtime = Runtime::new();
        let nested_ran = Rc::new(Cell::new(false));
        let nested_marker = nested_ran.clone();

        runtime.spawn(async move {
            spawn(async move {
                nested_marker.set(true);
            });
        });
        runtime.poll();
        assert!(!nested_ran.get());
        runtime.poll();
        assert!(nested_ran.get());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn each_runtime_wakes_only_its_own_host_event_loop() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct CountingWaker(Arc<AtomicUsize>);

        impl ExecutorWaker for CountingWaker {
            fn wake(&self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }

        let _guard = test_guard();
        let first_wakes = Arc::new(AtomicUsize::new(0));
        let second_wakes = Arc::new(AtomicUsize::new(0));
        let first = Runtime::new();
        let second = Runtime::new();
        first.set_executor_waker(Arc::new(CountingWaker(first_wakes.clone())));
        second.set_executor_waker(Arc::new(CountingWaker(second_wakes.clone())));

        first.spawn(async {});
        assert_eq!(first_wakes.load(Ordering::SeqCst), 1);
        assert_eq!(second_wakes.load(Ordering::SeqCst), 0);

        second.spawn(async {});
        assert_eq!(first_wakes.load(Ordering::SeqCst), 1);
        assert_eq!(second_wakes.load(Ordering::SeqCst), 1);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn task_woken_from_another_runtime_is_polled_only_by_its_owner() {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
            mpsc,
        };

        let _guard = test_guard();
        let completed = Arc::new(AtomicBool::new(false));
        let completed_on_owner = completed.clone();
        let (waker_tx, waker_rx) = mpsc::sync_channel::<Waker>(1);
        let (drained_tx, drained_rx) = mpsc::sync_channel::<()>(1);

        let owner = std::thread::spawn(move || {
            let runtime = Runtime::new();
            runtime.activate();

            struct ExternalWake {
                waker_tx: Option<mpsc::SyncSender<Waker>>,
            }

            impl Future for ExternalWake {
                type Output = ();

                fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
                    if let Some(sender) = self.waker_tx.take() {
                        sender.send(cx.waker().clone()).unwrap();
                        Poll::Pending
                    } else {
                        Poll::Ready(())
                    }
                }
            }

            runtime.spawn(async move {
                ExternalWake {
                    waker_tx: Some(waker_tx),
                }
                .await;
                completed_on_owner.store(true, Ordering::SeqCst);
            });
            runtime.poll();

            drained_rx.recv().unwrap();
            runtime.poll();
            runtime.cancel_all();
        });

        let task_waker = waker_rx.recv().unwrap();
        task_waker.wake_by_ref();

        // Simulate a second GUI runtime polling on another thread. The
        // current global queue loses the first runtime's task id here.
        poll();
        drained_tx.send(()).unwrap();
        owner.join().unwrap();

        assert!(completed.load(Ordering::SeqCst));
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
