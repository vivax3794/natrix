//! State traits

use std::cell::RefCell;
use std::rc::Rc;

use crate::dom::element::{
    DynElement,
    Element,
    ElementRenderResult,
    MaybeStaticElement,
    generate_fallback_node,
};
use crate::error_handling::log_or_panic;
use crate::get_document;
use crate::reactivity::KeepAlive;
use crate::reactivity::render_callbacks::RenderingState;
use crate::reactivity::state::{Ctx, EventToken, HookKey, State, messages};

// TODO: Allow storing sub component senders on the state.
// We would need derive macro support to mark a field as non-reactive.

/// The result of rendering a component
///
/// This should be kept in memory for as long as the component is in the dom.
#[must_use = "Dropping this before the component is unmounted will cause panics"]
#[expect(
    dead_code,
    reason = "This is used to keep the component alive and we do not need to use it"
)]
pub struct RenderResult<C: State> {
    /// The component data
    data: Rc<RefCell<Ctx<C>>>,
    /// The various things that need to be kept alive
    keep_alive: Vec<KeepAlive>,
}

/// Mount the specified component at natrixses default location. and calls `setup_runtime`
/// This is what should be used when building with the natrix cli.
///
/// IMPORTANT: This is the intended entry point for `natrix-cli` build applications, and the natrix
/// cli build system expects this to be called. And you should not attempt to access browser ap
///
/// This method implicitly leaks the memory of the root component
///
/// # Panics
/// If the mount point is not found, which should never happen if using `natrix build`
#[expect(
    clippy::expect_used,
    reason = "This will never happen if `natrix build` is used, and also happens early in the app lifecycle"
)]
pub fn mount<C: State>(component: C, tree: impl Element<C>) {
    crate::panics::set_panic_hook();
    #[cfg(feature = "console_log")]
    if cfg!(target_arch = "wasm32") {
        if let Err(err) = console_log::init_with_level(log::Level::Trace) {
            crate::error_handling::log_or_panic!("Failed to create logger: {err}");
        }
    }
    #[cfg(feature = "_internal_bundle")]
    if let Err(err) = simple_logger::init_with_level(log::Level::Trace) {
        eprintln!("Failed to setup logger {err}");
    }
    log::info!("Logging initialized");
    #[cfg(feature = "_internal_collect_css")]
    crate::css::do_css_setup();

    if cfg!(feature = "_internal_bundle") {
        log::info!("bundle mode, aboring mount.");
        return;
    }

    mount_at(component, tree, natrix_shared::MOUNT_POINT).expect("Failed to mount");
}

/// Mounts the component at the target id
/// Replacing the element with the component
///
/// This method implicitly leaks the memory of the root component
///
/// # Errors
/// If target mount point is not found.
pub fn mount_at<C: State>(
    component: C,
    tree: impl Element<C>,
    target_id: &'static str,
) -> Result<(), &'static str> {
    let result = render_component(component, tree, target_id)?;

    std::mem::forget(result);
    Ok(())
}

/// Mounts the component at the target id
/// Replacing the element with the component
/// # Errors
/// If target mount point is not found.
pub fn render_component<C: State>(
    component: C,
    tree: impl Element<C>,
    target_id: &str,
) -> Result<RenderResult<C>, &'static str> {
    log::info!(
        "Mounting root component {} at #{target_id}",
        std::any::type_name::<C>()
    );
    let data = Ctx::new(component);

    let mut borrow_data = data.borrow_mut();

    let mut keep_alive = Vec::new();
    let mut hooks = Vec::new();

    let mut state = RenderingState {
        keep_alive: &mut keep_alive,
        hooks: &mut hooks,
        parent_dep: HookKey::default(),
    };
    let node = tree
        .render()
        .render(&mut borrow_data, &mut state)
        .into_node();

    let document = get_document();
    let target = document
        .get_element_by_id(target_id)
        .ok_or("Failed to get mount point")?;
    target
        .replace_with_with_node_1(&node)
        .map_err(|_| "Failed to replace mount point")?;

    drop(borrow_data);

    Ok(RenderResult { data, keep_alive })
}
