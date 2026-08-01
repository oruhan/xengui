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
use std::sync::atomic::{ AtomicU64, Ordering };
use std::sync::{ Arc, Mutex };
use std::task::{ Context, Poll, Wake, Waker };

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
    fn wake(&self);
}

#[cfg(target_arch = "wasm32")]
pub trait ExecutorWaker {
    fn wake(&self);
}

#[cfg(not(target_arch = "wasm32"))]
static EXECUTOR_WAKER: Mutex<Option<Arc<dyn ExecutorWaker>>> = Mutex::new(None);

#[cfg(target_arch = "wasm32")]
thread_local! {
    static EXECUTOR_WAKER: RefCell<Option<Arc<dyn ExecutorWaker>>> = const { RefCell::new(None) };
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_executor_waker(waker: Arc<dyn ExecutorWaker>) {
    *EXECUTOR_WAKER.lock().unwrap() = Some(waker);
}

#[cfg(target_arch = "wasm32")]
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
pub fn spawn<F>(future: F) where F: Future + 'static {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

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
}
