use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};
use xengui::{RuntimeContext, dom, hooks, style::theme};

struct Redraw(Rc<Cell<usize>>);
impl xengui::RedrawRequester for Redraw {
    fn request_redraw(&self) {
        self.0.set(self.0.get() + 1);
    }
}

struct Pending {
    dropped: Rc<Cell<usize>>,
    waker: Rc<RefCell<Option<Waker>>>,
}
impl Future for Pending {
    type Output = Result<usize, ()>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        *self.waker.borrow_mut() = Some(cx.waker().clone());
        Poll::Pending
    }
}
impl Drop for Pending {
    fn drop(&mut self) {
        self.dropped.set(self.dropped.get() + 1);
    }
}

fn commit<R>(build: impl FnOnce() -> R) -> R {
    hooks::begin_render();
    let result = hooks::component("same-key", build);
    hooks::end_render();
    hooks::run_pending_effects();
    result
}

#[test]
fn same_thread_contexts_isolate_services_and_setters_target_their_owner() {
    let a = RuntimeContext::new();
    let b = RuntimeContext::new();
    let redraw_a = Rc::new(Cell::new(0));
    let redraw_b = Rc::new(Cell::new(0));
    let setter;
    {
        let _guard = a.enter();
        theme::set_current_theme(xengui::Theme::dark());
        theme::set_active_theme(3);
        xengui::style::length::set_viewport_size(300., 400.);
        xengui::style::responsive::set_current_breakpoint_from_width(300.);
        xengui::devtools::set_enabled(true);
        hooks::set_redraw_handle(Rc::new(Redraw(redraw_a.clone())));
        setter = commit(|| hooks::use_state(10).1);
        dom::click("shared-id");
    }
    {
        let _guard = b.enter();
        theme::set_current_theme(xengui::Theme::light());
        xengui::style::length::set_viewport_size(1400., 900.);
        xengui::style::responsive::set_current_breakpoint_from_width(1400.);
        hooks::set_redraw_handle(Rc::new(Redraw(redraw_b.clone())));
        assert!(!xengui::devtools::is_enabled());
        assert!(theme::take_theme_switch().is_none());
        assert!(dom::take_actions("shared-id").is_empty());
        assert_eq!(commit(|| hooks::use_state(20).0), 20);
        setter.set(11);
        assert_eq!(theme::current_theme(), xengui::Theme::light());
        assert!(!hooks::take_dirty());
        assert_eq!(redraw_b.get(), 0);
    }
    {
        let _guard = a.enter();
        assert_eq!(theme::current_theme(), xengui::Theme::dark());
        assert_eq!(xengui::style::length::viewport_size(), (300., 400.));
        assert_eq!(xengui::current_breakpoint(), xengui::Breakpoint::Compact);
        assert!(theme::take_theme_switch().is_some());
        assert_eq!(dom::take_actions("shared-id"), vec![dom::DomAction::Click]);
        assert_eq!(commit(|| hooks::use_state(0).0), 11);
        assert!(hooks::take_dirty());
        assert!(redraw_a.get() >= 2);
    }
}

#[test]
fn tasks_and_bound_callbacks_restore_context_and_route_dom_to_their_owner() {
    let a = RuntimeContext::new();
    let b = RuntimeContext::new();
    let callback = a.bind(|()| dom::click("callback"));
    {
        let _guard = a.enter();
        xengui::task::spawn(async {
            dom::click("task");
            xengui::task::spawn(async {
                dom::click("nested");
            });
        });
    }
    let _guard = b.enter();
    callback(());
    a.tasks().poll();
    b.tasks().poll();
    assert!(dom::take_actions("task").is_empty());
    assert!(dom::take_actions("callback").is_empty());
    {
        let _guard = a.enter();
        assert_eq!(dom::take_actions("task").len(), 1);
        assert_eq!(dom::take_actions("callback").len(), 1);
        assert!(dom::take_actions("nested").is_empty());
    }
    a.tasks().poll();
    {
        let _guard = a.enter();
        assert_eq!(dom::take_actions("nested").len(), 1);
    }
    drop(a);
    callback(());
    assert!(dom::take_actions("callback").is_empty());
}

#[test]
fn unmount_cancels_sleeping_resources_and_invalidates_old_handles() {
    let runtime = RuntimeContext::new();
    let _guard = runtime.enter();
    let dropped = Rc::new(Cell::new(0));
    let waker = Rc::new(RefCell::new(None));
    let resource = commit(|| {
        hooks::use_resource_once({
            let dropped = dropped.clone();
            let waker = waker.clone();
            move || Pending {
                dropped: dropped.clone(),
                waker: waker.clone(),
            }
        })
    });
    runtime.tasks().poll();
    assert_eq!(dropped.get(), 0);
    hooks::begin_render();
    hooks::end_render();
    hooks::run_pending_effects();
    assert_eq!(
        dropped.get(),
        1,
        "unmount must drop a future that never wakes"
    );
    hooks::take_dirty();
    resource.refresh();
    resource.invalidate();
    waker.borrow().as_ref().unwrap().wake_by_ref();
    runtime.tasks().poll();
    assert!(!hooks::take_dirty());
    assert_eq!(dropped.get(), 1);
}

#[test]
fn refresh_and_invalidate_drop_superseded_resource_futures() {
    let runtime = RuntimeContext::new();
    let _guard = runtime.enter();
    let dropped = Rc::new(Cell::new(0));
    let resource = commit(|| {
        hooks::use_resource_once({
            let dropped = dropped.clone();
            move || Pending {
                dropped: dropped.clone(),
                waker: Rc::default(),
            }
        })
    });
    runtime.tasks().poll();
    resource.refresh();
    assert_eq!(dropped.get(), 1);
    runtime.tasks().poll();
    resource.invalidate();
    assert_eq!(dropped.get(), 2);
}

#[test]
fn dropping_context_drops_tasks_pending_effects_and_runs_mounted_cleanup() {
    let runtime = RuntimeContext::new();
    let dropped = Rc::new(Cell::new(0));
    let cleaned = Rc::new(Cell::new(0));
    let waker = Rc::new(RefCell::new(None));
    let setter;
    {
        let _guard = runtime.enter();
        setter = commit(|| {
            hooks::use_effect(
                {
                    let cleaned = cleaned.clone();
                    move || move || cleaned.set(cleaned.get() + 1)
                },
                (),
            );
            hooks::use_resource_once({
                let dropped = dropped.clone();
                let waker = waker.clone();
                move || Pending {
                    dropped: dropped.clone(),
                    waker: waker.clone(),
                }
            });
            hooks::use_state(0).1
        });
        runtime.tasks().spawn(Pending {
            dropped: dropped.clone(),
            waker: Rc::default(),
        });
        runtime.tasks().poll();
        hooks::begin_render();
        hooks::component("uncommitted", || {
            let marker = Pending {
                dropped: dropped.clone(),
                waker: Rc::default(),
            };
            hooks::use_effect(
                move || -> () {
                    drop(marker);
                    panic!("uncommitted effect ran");
                },
                (),
            );
        });
    }
    let weak = Rc::downgrade(&runtime);
    drop(runtime);
    assert!(weak.upgrade().is_none());
    assert_eq!(dropped.get(), 3);
    assert_eq!(cleaned.get(), 1);
    setter.set(99);
    waker.borrow().as_ref().unwrap().wake_by_ref();
}

#[test]
fn nested_activation_restores_context_on_panic() {
    let a = RuntimeContext::new();
    let b = RuntimeContext::new();
    let _guard = a.enter();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = b.enter();
        panic!("test unwind");
    }));
    assert!(result.is_err());
    assert!(Rc::ptr_eq(
        &RuntimeContext::current().upgrade().unwrap(),
        &a
    ));
}

#[test]
fn cleanup_during_drop_cannot_write_into_another_active_context() {
    let a = RuntimeContext::new();
    let b = RuntimeContext::new();
    {
        let _guard = a.enter();
        commit(|| {
            hooks::use_effect(
                || {
                    || {
                        dom::click("cleanup");
                        xengui::task::spawn(async {
                            panic!("shutdown task must not run");
                        });
                    }
                },
                (),
            )
        });
    }
    let _guard = b.enter();
    drop(a);
    assert!(dom::take_actions("cleanup").is_empty());
    assert!(!hooks::take_dirty());
    b.tasks().poll();
    assert!(Rc::ptr_eq(
        &RuntimeContext::current().upgrade().unwrap(),
        &b
    ));
}

#[test]
fn stale_setter_cannot_modify_a_remounted_component_with_the_same_key() {
    let runtime = RuntimeContext::new();
    let _guard = runtime.enter();
    let setter = commit(|| hooks::use_state(1).1);
    hooks::begin_render();
    hooks::run_pending_effects();
    commit(|| hooks::use_state(2));
    hooks::take_dirty();
    setter.set(3);
    assert_eq!(commit(|| hooks::use_state(0).0), 2);
    assert!(!hooks::take_dirty());
}

#[test]
fn resource_refresh_from_another_context_stays_on_its_owner() {
    let a = RuntimeContext::new();
    let b = RuntimeContext::new();
    let calls = Rc::new(Cell::new(0));
    let resource;
    {
        let _guard = a.enter();
        resource = commit(|| {
            hooks::use_resource_once({
                let calls = calls.clone();
                move || {
                    let calls = calls.clone();
                    async move {
                        calls.set(calls.get() + 1);
                        Ok::<_, ()>(7)
                    }
                }
            })
        });
    }
    let _guard = b.enter();
    resource.refresh();
    b.tasks().poll();
    assert_eq!(calls.get(), 0);
    a.tasks().poll();
    assert_eq!(calls.get(), 1);
    assert!(!hooks::take_dirty());
    let _guard = a.enter();
    assert!(hooks::take_dirty());
}

#[test]
fn cancelling_all_inside_poll_does_not_resurrect_the_current_future() {
    let runtime = RuntimeContext::new();
    let dropped = Rc::new(Cell::new(0));
    let marker = Pending {
        dropped: dropped.clone(),
        waker: Rc::default(),
    };
    runtime.tasks().spawn(async move {
        xengui::task::cancel_all();
        marker.await
    });
    runtime.tasks().poll();
    assert_eq!(dropped.get(), 1);
}

#[test]
fn event_context_and_platform_policy_remain_bound_to_their_runtime() {
    let a = RuntimeContext::new();
    let b = RuntimeContext::new();
    let services_a = Rc::new(xengui::HeadlessPlatformServices::default());
    let services_b = Rc::new(xengui::HeadlessPlatformServices::default());
    let event_a;
    {
        let _guard = a.enter();
        xengui::set_platform_services(services_a.clone());
        let mut policy = xengui::ripple::ripple_config();
        policy.enabled = false;
        xengui::set_ripple_config(policy);
        event_a = xengui::EventCtx::new();
    }
    let _guard = b.enter();
    xengui::set_platform_services(services_b.clone());
    assert!(xengui::ripple::ripple_config().enabled);
    event_a
        .platform_services()
        .clipboard()
        .write_text("A".into())
        .unwrap();
    xengui::EventCtx::new()
        .platform_services()
        .clipboard()
        .write_text("B".into())
        .unwrap();
    assert_eq!(services_a.clipboard_text().as_deref(), Some("A"));
    assert_eq!(services_b.clipboard_text().as_deref(), Some("B"));
    event_a.spawn(async {
        dom::click("event-task");
    });
    b.tasks().poll();
    assert!(dom::take_actions("event-task").is_empty());
    a.tasks().poll();
    {
        let _guard = a.enter();
        assert_eq!(dom::take_actions("event-task").len(), 1);
    }
    drop(a);
    event_a.spawn(async {
        panic!("disposed event owner must not spawn");
    });
    b.tasks().poll();
}
