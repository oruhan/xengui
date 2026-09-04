// SPDX-License-Identifier: Apache-2.0
use crate::RedrawRequester;
use smol_str::SmolStr;
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// Data and behavior represented by `ComponentId`.
pub struct ComponentId(SmolStr);

impl ComponentId {
    /// Returns or updates the `root` value.
    pub fn root() -> Self {
        Self(SmolStr::new("root"))
    }

    /// Returns or updates the `as_str` value.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Clone, Debug)]
/// Data and behavior represented by `ComponentKey`.
pub struct ComponentKey(SmolStr);

impl From<&str> for ComponentKey {
    fn from(v: &str) -> Self {
        Self(SmolStr::new(v))
    }
}

impl From<String> for ComponentKey {
    fn from(v: String) -> Self {
        Self(SmolStr::new(v))
    }
}

impl From<SmolStr> for ComponentKey {
    fn from(v: SmolStr) -> Self {
        Self(v)
    }
}

macro_rules! impl_component_key_from_int {
    ($($t:ty),*) => {
        $(
            impl From<$t> for ComponentKey {
                fn from(v: $t) -> Self {
                    Self(SmolStr::new(v.to_string()))
                }
            }
        )*
    };
}
impl_component_key_from_int!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

struct ComponentState {
    slots: Vec<Rc<RefCell<Box<dyn Any>>>>,
    cursor: usize,
}

impl ComponentState {
    fn new() -> Self {
        Self {
            slots: Vec::new(),
            cursor: 0,
        }
    }
}

thread_local! {
    static HOOK_STORE: RefCell<HashMap<ComponentId, ComponentState>> = RefCell::new(HashMap::new());

    static COMPONENT_STACK: RefCell<Vec<ComponentId>> = const { RefCell::new(Vec::new()) };

    static LIVE_COMPONENTS: RefCell<HashSet<ComponentId>> = RefCell::new(HashSet::new());

    static DIRTY: Cell<bool> = const { Cell::new(false) };

    static REDRAW_HANDLE: RefCell<Option<Rc<dyn RedrawRequester>>> = const { RefCell::new(None) };

    static RENDER_GENERATION: Cell<u64> = const { Cell::new(0) };

    static PENDING_EFFECTS: RefCell<Vec<PendingEffect>> = const { RefCell::new(Vec::new()) };
}

/// Returns or updates the `begin_render` value.
pub fn begin_render() {
    // Lets a render whose reconciliation gets superseded before finishing
    // be identified later, so its queued effects are never executed.
    RENDER_GENERATION.with(|g| g.set(g.get() + 1));

    // The live set belongs to the render that is about to start. Hook
    // state is pruned only after reconciliation commits, because composite
    // widgets register themselves during reconciliation rather than during
    // the root builder call.
    LIVE_COMPONENTS.with(|s| s.borrow_mut().clear());

    COMPONENT_STACK.with(|s| {
        let mut s = s.borrow_mut();
        debug_assert!(
            s.is_empty(),
            "xengui hooks: component stack is not empty - begin_render/end_render may have been called unevenly"
        );
        s.clear();
    });
}

/// Returns or updates the `end_render` value.
pub fn end_render() {
    // Pruning now happens at the start of the next begin_render, once the
    // previous cycle's LIVE set (which composite widgets only finish
    // populating well after this point) is complete.
}

// Runs (and clears) every effect cleanup left behind by a component that
// didn't appear in this render pass, since it will never build again.
fn run_unmount_cleanups(state: &ComponentState) {
    for slot in &state.slots {
        let cleanup = slot
            .borrow_mut()
            .downcast_mut::<EffectRecord>()
            .and_then(|record| record.cleanup.take());

        if let Some(cleanup) = cleanup {
            cleanup();
        }
    }
}

/// Returns or updates the `take_dirty` value.
pub fn take_dirty() -> bool {
    DIRTY.with(|d| d.replace(false))
}

/// Updates the `set_redraw_handle` value.
pub fn set_redraw_handle(handle: Rc<dyn RedrawRequester>) {
    REDRAW_HANDLE.with(|h| {
        *h.borrow_mut() = Some(handle);
    });
}

fn request_redraw() {
    REDRAW_HANDLE.with(|h| {
        if let Some(handle) = h.borrow().as_ref() {
            handle.request_redraw();
        }
    });
}

fn current_component_id() -> ComponentId {
    COMPONENT_STACK.with(|s| {
        s.borrow().last().cloned().unwrap_or_else(|| {
            panic!(
                "use_state: called outside a component() scope. \
                 use_state can only be used within App::render's root function or \
                 inside a component(key, ...) scope."
            )
        })
    })
}

fn push_component(key: ComponentKey) -> ComponentId {
    let id = COMPONENT_STACK.with(|s| match s.borrow().last() {
        Some(parent) => ComponentId(SmolStr::new(format!("{}\u{1f}{}", parent.as_str(), key.0))),
        None => ComponentId(key.0),
    });

    HOOK_STORE.with(|store| {
        let mut store = store.borrow_mut();
        let state = store.entry(id.clone()).or_insert_with(ComponentState::new);
        state.cursor = 0;
    });

    let first_time_this_frame = LIVE_COMPONENTS.with(|s| s.borrow_mut().insert(id.clone()));
    if !first_time_this_frame {
        let message = format!(
            "duplicate component key '{}' - used twice in the same frame. \
             In dynamic lists, give each item a unique key (like React's 'key' prop).",
            id.as_str()
        );
        log::warn!("xengui: {message}");
        crate::devtools::log_warning(id.as_str(), "component", message);
    }

    COMPONENT_STACK.with(|s| s.borrow_mut().push(id.clone()));
    id
}

fn pop_component() {
    COMPONENT_STACK.with(|s| {
        s.borrow_mut().pop();
    });
}

/// component(key, render).
///
/// ```ignore
/// let mut list_view = View::new().flex_direction(FlexDirection::Column);
/// for item in &items {
///     list_view = list_view.child(component(item.id, || {
///         let (checked, set_checked) = use_state(false);
///         Button::new()
///             .label(if checked { "✓" } else { "" })
///             .on_click(move |_ctx| set_checked.set(!checked))
///     }));
/// }
/// ```
pub fn component<R>(key: impl Into<ComponentKey>, render: impl FnOnce() -> R) -> R {
    push_component(key.into());
    let result = render();
    pop_component();
    result
}

/// Creates a state value that persists across component rebuilds.
///
/// `use_state` returns the current state value together with a setter that can
/// be used to update it. Calling [`SetState::set`] schedules the owning
/// component to be rebuilt with the new value.
///
/// State is preserved as long as the component's identity remains stable. When
/// rendering dynamic lists, assign a stable key to each component to ensure
/// that state is associated with the correct item.
///
/// ## Panics
///
/// Panics if the order of hook invocations changes between rebuilds. Hooks must
/// always be called unconditionally and in the same order on every rebuild.
///
/// ## Example
///
/// ```ignore
/// let (count, set_count) = use_state(0i32);
///
/// View::new().child(
///     Button::new()
///         .label(format!("Count: {count}"))
///         .on_click(move |_ctx| set_count.set(count + 1))
/// );
/// ```
pub fn use_state<T: Clone + 'static>(initial: T) -> (T, SetState<T>) {
    let id = current_component_id();

    let (slot, idx) = HOOK_STORE.with(|store| {
        let mut store = store.borrow_mut();
        let state = store
            .get_mut(&id)
            .expect("use_state: internal error - provided binding used without begin/push");

        let idx = state.cursor;
        state.cursor += 1;

        if idx == state.slots.len() {
            state
                .slots
                .push(Rc::new(RefCell::new(Box::new(initial) as Box<dyn Any>)));
        }

        (state.slots[idx].clone(), idx)
    });

    let value = {
        let borrowed = slot.borrow();
        borrowed
            .downcast_ref::<T>()
            .unwrap_or_else(|| {
                panic!(
                    "use_state: hook order broken in component '{}' (slot #{idx}) - do not call use_state conditionally (inside an if/loop). In dynamic lists, wrap each item in a component (e.g., component(key, ...)) to give it its own isolated hook order.",
                    id.as_str()
                )
            })
            .clone()
    };

    (
        value,
        SetState {
            slot,
            _marker: PhantomData,
        },
    )
}

/// Data and behavior represented by `SetState`.
pub struct SetState<T> {
    slot: Rc<RefCell<Box<dyn Any>>>,
    _marker: PhantomData<T>,
}

impl<T> Clone for SetState<T> {
    fn clone(&self) -> Self {
        Self {
            slot: self.slot.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: 'static> SetState<T> {
    /// Returns or updates the `set` value.
    pub fn set(&self, value: T) {
        *self.slot.borrow_mut() = Box::new(value);
        DIRTY.with(|d| d.set(true));
        if crate::devtools::is_enabled() {
            crate::devtools::log_rerender("?", "SetState", "state set");
        }
        request_redraw();
    }

    /// Returns or updates the `update` value.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        {
            let mut borrowed = self.slot.borrow_mut();
            let current = borrowed
                .downcast_mut::<T>()
                .expect("use_state: SetState<T> used with the wrong type");

            f(current);
        }
        DIRTY.with(|d| d.set(true));
        if crate::devtools::is_enabled() {
            crate::devtools::log_rerender("?", "SetState", "state updated");
        }
        request_redraw();
    }
}

/// Returns or updates the `mark_dirty_and_redraw` value.
pub fn mark_dirty_and_redraw() {
    DIRTY.with(|d| d.set(true));
    request_redraw();
}

fn current_generation() -> u64 {
    RENDER_GENERATION.with(Cell::get)
}

/// Object-safe equality for an effect's whole dependency list, so
/// `use_effect` can accept `()`, arrays, or slices without needing one
/// concrete type shared across every call site.
trait StoredDeps: Any {
    fn eq_dyn(&self, other: &dyn StoredDeps) -> bool;
    fn as_any(&self) -> &dyn Any;
}

impl<T: PartialEq + 'static> StoredDeps for T {
    fn eq_dyn(&self, other: &dyn StoredDeps) -> bool {
        other
            .as_any()
            .downcast_ref::<T>()
            .is_some_and(|o| self == o)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Opaque, erased snapshot of a `use_effect` dependency list, stored
/// between renders to decide whether the effect needs to run again.
pub struct DepsSnapshot(Box<dyn StoredDeps>);

fn deps_changed(old: &DepsSnapshot, new: &DepsSnapshot) -> bool {
    !old.0.eq_dyn(new.0.as_ref())
}

/// Converts a dependency list passed to [`use_effect`] into a comparable,
/// erased snapshot. Implemented for `()` (no dependencies - runs once),
/// owned arrays, and array/slice references.
pub trait EffectDeps {
    /// Returns or updates the `snapshot` value.
    fn snapshot(self) -> DepsSnapshot;
}

impl EffectDeps for () {
    fn snapshot(self) -> DepsSnapshot {
        DepsSnapshot(Box::new(()))
    }
}

impl<T: PartialEq + 'static, const N: usize> EffectDeps for [T; N] {
    fn snapshot(self) -> DepsSnapshot {
        DepsSnapshot(Box::new(self))
    }
}

impl<T: PartialEq + Clone + 'static, const N: usize> EffectDeps for &[T; N] {
    fn snapshot(self) -> DepsSnapshot {
        DepsSnapshot(Box::new(self.clone()))
    }
}

impl<T: PartialEq + Clone + 'static> EffectDeps for &[T] {
    fn snapshot(self) -> DepsSnapshot {
        DepsSnapshot(Box::new(self.to_vec()))
    }
}

/// Normalizes a `use_effect` closure's return value: `()` means no
/// cleanup, anything callable once becomes the cleanup that runs before
/// the next execution (or on unmount).
pub trait EffectCleanup {
    /// Returns or updates the `into_cleanup` value.
    fn into_cleanup(self) -> Option<Box<dyn FnOnce()>>;
}

impl EffectCleanup for () {
    fn into_cleanup(self) -> Option<Box<dyn FnOnce()>> {
        None
    }
}

impl<F: FnOnce() + 'static> EffectCleanup for F {
    fn into_cleanup(self) -> Option<Box<dyn FnOnce()>> {
        Some(Box::new(self))
    }
}

struct EffectRecord {
    deps: Option<DepsSnapshot>,
    cleanup: Option<Box<dyn FnOnce()>>,
    mounted: bool,
    pending: bool,
}

type BoxedEffectFn = Box<dyn FnOnce() -> Option<Box<dyn FnOnce()>>>;

struct PendingEffect {
    slot: Rc<RefCell<Box<dyn Any>>>,
    new_deps: DepsSnapshot,
    run: BoxedEffectFn,
    generation: u64,
}

/// Runs a side effect after this component's tree has actually been
/// committed, similar to React's `useEffect`.
///
/// `effect` runs once after the first successful render, and again
/// whenever `deps` changes value (compared by equality, not identity).
/// Returning a closure from `effect` registers it as cleanup, run right
/// before the next execution and when the component is unmounted.
///
/// ## Panics
///
/// Panics if called outside a `component()` scope, or if the order of
/// hook invocations changes between rebuilds (see [`use_state`]).
pub fn use_effect<F, R, D>(effect: F, deps: D)
where
    F: FnOnce() -> R + 'static,
    R: EffectCleanup + 'static,
    D: EffectDeps,
{
    let id = current_component_id();
    let new_deps = deps.snapshot();

    let (slot, idx) = HOOK_STORE.with(|store| {
        let mut store = store.borrow_mut();
        let state = store
            .get_mut(&id)
            .expect("use_effect: internal error - provided binding used without begin/push");

        let idx = state.cursor;
        state.cursor += 1;

        if idx == state.slots.len() {
            state
                .slots
                .push(Rc::new(RefCell::new(Box::new(EffectRecord {
                    deps: None,
                    cleanup: None,
                    mounted: false,
                    pending: false,
                }) as Box<dyn Any>)));
        }

        (state.slots[idx].clone(), idx)
    });

    let should_run = {
        let borrowed = slot.borrow();
        let record = borrowed
            .downcast_ref::<EffectRecord>()
            .unwrap_or_else(|| {
                panic!(
                    "use_effect: hook order broken in component '{}' (slot #{idx}) - do not call use_effect conditionally.",
                    id.as_str()
                )
            });

        match &record.deps {
            None => true,
            Some(old_deps) => deps_changed(old_deps, &new_deps),
        }
    };

    if !should_run {
        return;
    }

    slot.borrow_mut()
        .downcast_mut::<EffectRecord>()
        .expect("use_effect: internal error")
        .pending = true;

    let run: BoxedEffectFn = Box::new(move || effect().into_cleanup());

    PENDING_EFFECTS.with(|q| {
        q.borrow_mut().push(PendingEffect {
            slot,
            new_deps,
            run,
            generation: current_generation(),
        });
    });
}

/// Runs every effect queued by the render that was just committed to the
/// tree. Must be called after reconciliation completes, never during
/// widget building - the reconciler's own commit point is the intended
/// caller.
pub fn run_pending_effects() {
    let generation = current_generation();
    let pending = PENDING_EFFECTS.with(|q| std::mem::take(&mut *q.borrow_mut()));

    // Reconciliation has committed at this point, so LIVE_COMPONENTS now
    // contains every root and composite component in the committed tree.
    // Removing hook state here makes unmount cleanup happen in the same
    // commit that removed the component, without pruning composites before
    // reconciliation has had a chance to render them.
    LIVE_COMPONENTS.with(|live| {
        let live = live.borrow();
        HOOK_STORE.with(|store| {
            store.borrow_mut().retain(|id, state| {
                let keep = live.contains(id);
                if !keep {
                    run_unmount_cleanups(state);
                }
                keep
            });
        });
    });

    for entry in pending {
        // An effect queued by a render that got superseded before its
        // reconciliation finished was never actually committed, so it
        // must not run.
        if entry.generation != generation {
            continue;
        }

        let old_cleanup = {
            let mut boxed = entry.slot.borrow_mut();
            let record = boxed
                .downcast_mut::<EffectRecord>()
                .expect("use_effect: internal error");
            record.pending = false;
            record.cleanup.take()
        };

        if let Some(cleanup) = old_cleanup {
            cleanup();
        }

        let new_cleanup = (entry.run)();

        let mut boxed = entry.slot.borrow_mut();
        let record = boxed
            .downcast_mut::<EffectRecord>()
            .expect("use_effect: internal error");
        record.deps = Some(entry.new_deps);
        record.cleanup = new_cleanup;
        record.mounted = true;
    }
}

// ---------------------------------------------------------------------
// use_resource: combines use_state + use_effect + the task executor into
// a single hook for async data loading with loading/error tracking,
// dependency-driven reloads, and manual refresh/invalidate.
// ---------------------------------------------------------------------

use std::future::Future;

/// Snapshot of an async resource's current lifecycle state.
pub enum ResourceState<T, E> {
    /// The `Idle` variant.
    Idle,
    /// The `Loading` variant.
    Loading,
    /// The `Ready` variant.
    Ready(T),
    /// The `Error` variant.
    Error(E),
}

impl<T, E> ResourceState<T, E> {
    /// Returns or updates the `data` value.
    pub fn data(&self) -> Option<&T> {
        match self {
            Self::Ready(value) => Some(value),
            _ => None,
        }
    }

    /// Returns or updates the `error` value.
    pub fn error(&self) -> Option<&E> {
        match self {
            Self::Error(err) => Some(err),
            _ => None,
        }
    }

    /// Returns whether the `is_loading` condition is satisfied.
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }
}

impl<T: Clone, E: Clone> Clone for ResourceState<T, E> {
    fn clone(&self) -> Self {
        match self {
            Self::Idle => Self::Idle,
            Self::Loading => Self::Loading,
            Self::Ready(v) => Self::Ready(v.clone()),
            Self::Error(e) => Self::Error(e.clone()),
        }
    }
}

/// Handle returned by [`use_resource`]. `refresh`/`invalidate` stay valid
/// for as long as the owning component is mounted, even when captured by
/// a `move` closure (e.g. a button's `on_click`).
pub struct Resource<T, E> {
    state: ResourceState<T, E>,
    do_refresh: Rc<dyn Fn()>,
    do_invalidate: Rc<dyn Fn()>,
}

impl<T, E> Resource<T, E> {
    /// Returns or updates the `data` value.
    pub fn data(&self) -> Option<&T> {
        self.state.data()
    }

    /// Returns or updates the `error` value.
    pub fn error(&self) -> Option<&E> {
        self.state.error()
    }

    /// Returns or updates the `loading` value.
    pub fn loading(&self) -> bool {
        self.state.is_loading()
    }

    /// Returns whether the `has_value` condition is satisfied.
    pub fn has_value(&self) -> bool {
        self.data().is_some()
    }

    /// Returns whether the `has_error` condition is satisfied.
    pub fn has_error(&self) -> bool {
        self.error().is_some()
    }

    /// Returns or updates the `state` value.
    pub fn state(&self) -> &ResourceState<T, E> {
        &self.state
    }

    /// Reruns the loader with the most recently seen dependency value,
    /// regardless of whether that value actually changed.
    pub fn refresh(&self) {
        (self.do_refresh)();
    }

    /// Clears the resource back to `Idle` without reloading.
    pub fn invalidate(&self) {
        (self.do_invalidate)();
    }
}

impl<T: Clone, E: Clone> Clone for Resource<T, E> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            do_refresh: self.do_refresh.clone(),
            do_invalidate: self.do_invalidate.clone(),
        }
    }
}

// Bumps the shared generation counter, flips the resource into Loading,
// and spawns the future on xengui's own task executor. A completion is
// only applied if `gen_cell` still holds the generation this load was
// started with, so a superseded load (newer deps, or a manual refresh)
// can't clobber fresher data.
fn spawn_resource_load<D, T, E, LF, Fut>(
    gen_cell: Rc<Cell<u64>>,
    loader: LF,
    deps: D,
    set_state: SetState<ResourceState<T, E>>,
) where
    D: 'static,
    T: 'static,
    E: 'static,
    LF: Fn(D) -> Fut + 'static,
    Fut: Future<Output = Result<T, E>> + 'static,
{
    let my_generation = gen_cell.get() + 1;
    gen_cell.set(my_generation);
    set_state.set(ResourceState::Loading);

    crate::task::spawn(async move {
        let result = loader(deps).await;
        if gen_cell.get() != my_generation {
            return;
        }
        set_state.set(match result {
            Ok(value) => ResourceState::Ready(value),
            Err(err) => ResourceState::Error(err),
        });
    });
}

/// Loads async data with automatic loading/error tracking, reloading
/// whenever `deps_fn`'s return value changes (compared by `PartialEq`,
/// like `use_effect`'s dependency list).
///
/// ```ignore
/// let github = use_resource(
///     || username.clone(),
///     |username| async move { fetch_user(username).await },
/// );
/// ```
///
/// ## Panics
///
/// Panics if called outside a `component()` scope, or if the order of
/// hook invocations changes between rebuilds (see [`use_state`]).
pub fn use_resource<D, T, E, DF, LF, Fut>(deps_fn: DF, loader: LF) -> Resource<T, E>
where
    D: PartialEq + Clone + 'static,
    T: Clone + 'static,
    E: Clone + 'static,
    DF: Fn() -> D,
    LF: Fn(D) -> Fut + Clone + 'static,
    Fut: Future<Output = Result<T, E>> + 'static,
{
    let deps = deps_fn();

    let (state, set_state) = use_state(ResourceState::<T, E>::Idle);

    // Rc<Cell<_>>/Rc<RefCell<_>> act as persistent refs here: their
    // identity survives rebuilds since the setter is never called, so
    // mutating their contents in place doesn't itself trigger a rebuild.
    let (gen_cell, _) = use_state(Rc::new(Cell::new(0u64)));
    let (deps_cell, _) = use_state(Rc::new(RefCell::new(deps.clone())));
    *deps_cell.borrow_mut() = deps.clone();

    use_effect(
        {
            let gen_cell = gen_cell.clone();
            let loader = loader.clone();
            let set_state = set_state.clone();
            let deps = deps.clone();
            move || {
                spawn_resource_load(gen_cell, loader, deps, set_state);
            }
        },
        [deps],
    );

    let do_refresh: Rc<dyn Fn()> = {
        let gen_cell = gen_cell.clone();
        let loader = loader.clone();
        let set_state = set_state.clone();
        let deps_cell = deps_cell.clone();
        Rc::new(move || {
            let deps = deps_cell.borrow().clone();
            spawn_resource_load(gen_cell.clone(), loader.clone(), deps, set_state.clone());
        })
    };

    let do_invalidate: Rc<dyn Fn()> = {
        let gen_cell = gen_cell.clone();
        let set_state = set_state.clone();
        Rc::new(move || {
            gen_cell.set(gen_cell.get() + 1);
            set_state.set(ResourceState::Idle);
        })
    };

    Resource {
        state,
        do_refresh,
        do_invalidate,
    }
}

/// Sugar for [`use_resource`] when the loader has no dependencies - runs
/// once on mount and only reloads via an explicit `refresh()`.
pub fn use_resource_once<T, E, LF, Fut>(loader: LF) -> Resource<T, E>
where
    T: Clone + 'static,
    E: Clone + 'static,
    LF: Fn() -> Fut + Clone + 'static,
    Fut: Future<Output = Result<T, E>> + 'static,
{
    use_resource(|| (), move |()| loader())
}

#[cfg(test)]
mod effect_tests {
    use super::*;

    #[test]
    fn runs_once_on_mount_and_skips_unchanged_deps() {
        let log = Rc::new(RefCell::new(Vec::<String>::new()));

        let build = || {
            component("effect_mount_root", || {
                let log = log.clone();
                use_effect(
                    move || {
                        log.borrow_mut().push("mount".to_string());
                    },
                    (),
                );
            });
        };

        begin_render();
        build();
        end_render();
        run_pending_effects();

        begin_render();
        build();
        end_render();
        run_pending_effects();

        assert_eq!(*log.borrow(), vec!["mount".to_string()]);
    }

    #[test]
    fn reruns_when_deps_change() {
        let log = Rc::new(RefCell::new(Vec::<String>::new()));

        let build = |value: i32| {
            component("effect_deps_root", || {
                let log = log.clone();
                use_effect(
                    move || {
                        log.borrow_mut().push(format!("run:{value}"));
                    },
                    [value],
                );
            });
        };

        begin_render();
        build(1);
        end_render();
        run_pending_effects();

        begin_render();
        build(1);
        end_render();
        run_pending_effects();

        begin_render();
        build(2);
        end_render();
        run_pending_effects();

        assert_eq!(
            *log.borrow(),
            vec!["run:1".to_string(), "run:2".to_string()]
        );
    }

    #[test]
    fn cleanup_runs_before_rerun_and_on_unmount() {
        let log = Rc::new(RefCell::new(Vec::<String>::new()));

        let build_child = |value: i32| {
            component("effect_cleanup_child", || {
                let log = log.clone();
                use_effect(
                    move || {
                        log.borrow_mut().push(format!("run:{value}"));
                        move || {
                            log.borrow_mut().push(format!("cleanup:{value}"));
                        }
                    },
                    [value],
                );
            });
        };

        begin_render();
        component("effect_cleanup_root", || build_child(1));
        end_render();
        run_pending_effects();

        begin_render();
        component("effect_cleanup_root", || build_child(2));
        end_render();
        run_pending_effects();

        assert_eq!(
            *log.borrow(),
            vec![
                "run:1".to_string(),
                "cleanup:1".to_string(),
                "run:2".to_string()
            ]
        );

        // Third render omits the child entirely, so its cleanup must fire
        // once during end_render's unmount pass.
        begin_render();
        component("effect_cleanup_root", || {});
        end_render();
        run_pending_effects();

        assert_eq!(
            *log.borrow(),
            vec![
                "run:1".to_string(),
                "cleanup:1".to_string(),
                "run:2".to_string(),
                "cleanup:2".to_string()
            ]
        );
    }
}
