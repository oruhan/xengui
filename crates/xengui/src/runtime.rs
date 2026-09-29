// SPDX-License-Identifier: Apache-2.0
//! Per-application services. Thread-local storage only selects the active owner.
//!
//! Hosts must enter their context for rendering, event dispatch and service
//! configuration. `xenframe::App` does this automatically. Standalone widgets
//! can read default service values without a context; writes in that mode are
//! transient and are not inherited by subsequently created applications.
//!
//! ```
//! let runtime = xengui::RuntimeContext::new();
//! let _guard = runtime.enter();
//! xengui::dom::click("button");
//! runtime.tasks().poll();
//! ```
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

thread_local! {
    static ACTIVE: RefCell<Weak<RuntimeContext>> = const { RefCell::new(Weak::new()) };
}

/// Owns the services and asynchronous work of one application tree.
pub struct RuntimeContext {
    states: RefCell<HashMap<TypeId, Rc<dyn Any>>>,
    tasks: crate::task::Runtime,
}

impl RuntimeContext {
    /// Creates an independent context on the GUI thread.
    pub fn new() -> Rc<Self> {
        Rc::new_cyclic(|owner| Self {
            states: RefCell::new(HashMap::new()),
            tasks: crate::task::Runtime::with_context(owner.clone()),
        })
    }

    /// Activates this context until the returned guard is dropped.
    pub fn enter(self: &Rc<Self>) -> RuntimeGuard {
        let previous = ACTIVE.with(|active| active.replace(Rc::downgrade(self)));
        RuntimeGuard {
            previous,
            _owner: Some(self.clone()),
        }
    }

    /// Temporarily clears the active context. Used by platform callbacks whose
    /// owning application has already been disposed, and by standalone executors.
    pub fn suspend() -> RuntimeGuard {
        let previous = ACTIVE.with(|active| active.replace(Weak::new()));
        RuntimeGuard {
            previous,
            _owner: None,
        }
    }

    /// Returns this context's task executor.
    pub fn tasks(&self) -> &crate::task::Runtime {
        &self.tasks
    }

    /// Captures the active context without extending its lifetime.
    pub fn current() -> Weak<Self> {
        ACTIVE.with(|active| active.borrow().clone())
    }

    /// Returns an application-owned service, initializing it on first use.
    pub fn state<T: Default + 'static>(&self) -> Rc<T> {
        let mut states = self.states.borrow_mut();
        states
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Rc::new(T::default()))
            .clone()
            .downcast()
            .ok()
            .unwrap()
    }
}

impl Drop for RuntimeContext {
    fn drop(&mut self) {
        // A destructor may run while another application is active. Never let
        // cleanup's implicit service calls leak into that application.
        let _restore = Self::suspend();
        let _tasks = self.tasks.activate();
        self.tasks.close();
        // Hook state performs effect cleanup before the other services disappear.
        self.states
            .get_mut()
            .remove(&TypeId::of::<crate::hooks::RuntimeState>());
        self.states.get_mut().clear();
    }
}

/// Restores the previous context, including during unwinding.
#[must_use = "keep the guard alive for the duration of runtime work"]
pub struct RuntimeGuard {
    previous: Weak<RuntimeContext>,
    _owner: Option<Rc<RuntimeContext>>,
}
impl Drop for RuntimeGuard {
    fn drop(&mut self) {
        ACTIVE.with(|active| {
            active.replace(self.previous.clone());
        });
    }
}

impl RuntimeContext {
    /// Wraps a callback with weak ownership and scoped activation. Calls after
    /// context disposal are ignored.
    pub fn bind<A: 'static>(
        self: &Rc<Self>,
        callback: impl Fn(A) + 'static,
    ) -> impl Fn(A) + 'static {
        let owner = Rc::downgrade(self);
        move |args| {
            if let Some(owner) = owner.upgrade() {
                let _guard = owner.enter();
                callback(args);
            }
        }
    }
}

// Module-local typed service groups preserve privacy and avoid a central list of
// widget implementation types. The constants below are accessors, never owners.
#[doc(hidden)]
#[macro_export]
macro_rules! runtime_state {
    ($( $(#[$attr:meta])* static $name:ident: $ty:ty = $init:expr; )*) => {
        #[allow(non_snake_case)]
        pub(crate) struct RuntimeState { $($name: $ty,)* }
        impl Default for RuntimeState {
            fn default() -> Self { Self { $($name: $init,)* } }
        }
        $(
            const $name: $crate::runtime::RuntimeSlot<RuntimeState, $ty> =
                $crate::runtime::RuntimeSlot { get: |state| &state.$name };
        )*
    };
}
pub(crate) use runtime_state;

/// Typed, stateless accessor used by runtime_state! in host crates.
#[doc(hidden)]
pub struct RuntimeSlot<S, T> {
    pub get: fn(&S) -> &T,
}
impl<S: Default + 'static, T> RuntimeSlot<S, T> {
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        if let Some(owner) = RuntimeContext::current().upgrade() {
            f((self.get)(&owner.state::<S>()))
        } else {
            f((self.get)(&S::default()))
        }
    }
}
