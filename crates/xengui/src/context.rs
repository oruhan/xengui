// SPDX-License-Identifier: Apache-2.0
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

thread_local! {
    static CONTEXT_STACK: RefCell<HashMap<TypeId, Vec<Rc<dyn Any>>>> = RefCell::new(HashMap::new());
}

// Pops this context's value when the enclosing scope ends, restoring
// whatever value (if any) the parent scope had provided.
/// Data and behavior represented by `ContextGuard`.
pub struct ContextGuard {
    type_id: TypeId,
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        CONTEXT_STACK.with(|stack| {
            if let Some(values) = stack.borrow_mut().get_mut(&self.type_id) {
                values.pop();
            }
        });
    }
}

/// Returns or updates the `provide_context` value.
pub fn provide_context<T: 'static>(value: T) -> ContextGuard {
    let type_id = TypeId::of::<T>();
    CONTEXT_STACK.with(|stack| {
        stack
            .borrow_mut()
            .entry(type_id)
            .or_default()
            .push(Rc::new(value));
    });
    ContextGuard { type_id }
}

/// Returns or updates the `use_context` value.
pub fn use_context<T: 'static>() -> Option<Rc<T>> {
    let type_id = TypeId::of::<T>();
    CONTEXT_STACK
        .with(|stack| {
            stack
                .borrow()
                .get(&type_id)
                .and_then(|values| values.last())
                .cloned()
        })
        .and_then(|rc| rc.downcast::<T>().ok())
}

// React's <Context.Provider value={...}>{children}</Context.Provider>
// equivalent: value is visible to use_context() calls made anywhere
// inside `f`, including nested component() scopes, and is automatically
// removed once `f` returns.
/// Updates the `with_context` value.
pub fn with_context<T: 'static, R>(value: T, f: impl FnOnce() -> R) -> R {
    let _guard = provide_context(value);
    f()
}
