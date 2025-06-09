//! Implements the reactive hooks for updating the dom in response to signal changessz.

use std::borrow::Cow;

use wasm_bindgen::JsCast;

use crate::dom::attributes::AttributeResult;
use crate::dom::classes::ClassResult;
use crate::dom::element::{ElementRenderResult, MaybeStaticElement, generate_fallback_node};
use crate::dom::{ToAttribute, ToClass};
use crate::error_handling::{debug_expect, debug_panic};
use crate::get_document;
use crate::reactivity::component::Component;
use crate::reactivity::signal::{ReactiveHook, RenderingState, UpdateResult};
use crate::reactivity::state::{HookKey, KeepAlive, RenderCtx, State};

/// A noop hook used to fill the `Rc<RefCell<...>>` while the initial render pass runs so that that
/// a real hook can be swapped in once initialized
pub(crate) struct DummyHook;
impl<C: Component> ReactiveHook<C> for DummyHook {
    fn update(&mut self, _ctx: &mut State<C>, _you: HookKey) -> UpdateResult {
        UpdateResult::Nothing
    }
    fn drop_us(self: Box<Self>) -> Vec<HookKey> {
        Vec::new()
    }
}

/// Reactive hook for swapping out a entire dom node.
pub(crate) struct ReactiveNode<C: Component> {
    /// The callback to produce nodes
    callback: Box<dyn Fn(&mut RenderCtx<C>) -> MaybeStaticElement<C>>,
    /// The current rendered node to replace
    target_node: web_sys::Node,
    /// Vector of various objects to be kept alive for the duration of the rendered content
    keep_alive: Vec<KeepAlive>,
    /// Hooks that are a child of this
    hooks: Vec<HookKey>,
}

impl<C: Component> ReactiveNode<C> {
    /// Render this hook and simply return the node
    ///
    /// IMPORTANT: This function works with the assumption what it returns will be put in its
    /// `target_node` field. This function is split out to facilitate `Self::create_initial`
    fn render(&mut self, ctx: &mut State<C>, you: HookKey) -> ElementRenderResult {
        ctx.clear();

        let element = (self.callback)(&mut RenderCtx {
            ctx,
            render_state: RenderingState {
                keep_alive: &mut self.keep_alive,
                hooks: &mut self.hooks,
                parent_dep: you,
            },
        });
        ctx.reg_dep(you);

        let mut state = RenderingState {
            keep_alive: &mut self.keep_alive,
            hooks: &mut self.hooks,
            parent_dep: you,
        };

        element.render(ctx, &mut state)
    }

    /// Create a new `ReactiveNode` registering the initial dependencies and returning both the `Rc`
    /// reference to it and the initial node (Which should be inserted in the dom)
    pub(crate) fn create_initial(
        callback: Box<dyn Fn(&mut RenderCtx<C>) -> MaybeStaticElement<C>>,
        ctx: &mut State<C>,
    ) -> (HookKey, web_sys::Node) {
        let me = ctx.insert_hook(Box::new(DummyHook));

        let Some(dummy_node) = get_document().body() else {
            debug_panic!("Document body not found");
            return (me, generate_fallback_node());
        };
        let dummy_node = dummy_node.into();

        let mut this = Self {
            callback,
            target_node: dummy_node,
            keep_alive: Vec::new(),
            hooks: Vec::new(),
        };
        let node = this.render(ctx, me).into_node();
        this.target_node = node.clone();
        ctx.set_hook(me, Box::new(this));

        (me, node)
    }

    /// Pulled out update method to facilite marking it as `default` on nightly
    fn update(&mut self, ctx: &mut State<C>, you: HookKey) -> UpdateResult {
        let hooks = std::mem::take(&mut self.hooks);
        let new_node = self.render(ctx, you);

        let new_node = match new_node {
            ElementRenderResult::Node(new_node) => new_node,
            ElementRenderResult::Text(new_text) => {
                if let Some(target_node) = self.target_node.dyn_ref::<web_sys::Text>() {
                    target_node.set_text_content(Some(&new_text));
                    return UpdateResult::DropHooks(hooks);
                }

                get_document().create_text_node(&new_text).into()
            }
        };

        let Some(parent) = self.target_node.parent_node() else {
            debug_panic!("Parent node of target node not found.");
            return UpdateResult::DropHooks(hooks);
        };

        debug_expect!(
            parent.replace_child(&new_node, &self.target_node),
            "Failed to replace parent"
        );
        self.target_node = new_node;

        UpdateResult::DropHooks(hooks)
    }
}

impl<C: Component> ReactiveHook<C> for ReactiveNode<C> {
    fn update(&mut self, ctx: &mut State<C>, you: HookKey) -> UpdateResult {
        self.update(ctx, you)
    }

    fn drop_us(self: Box<Self>) -> Vec<HookKey> {
        self.hooks
    }
}

/// A trait to allow `SimpleReactive` to deduplicate common reactive logic for attributes, classes,
/// styles, etc
pub(crate) trait ReactiveValue<C: Component> {
    /// Any potential state needed to apply the change
    type State: Default;

    /// Actually apply the change
    fn apply(
        self,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
        node: &web_sys::Element,
        state: &mut Self::State,
    );
}

/// A common wrapper for simple reactive operations to deduplicate dependency tracking code
pub(crate) struct SimpleReactive<C: Component, K: ReactiveValue<C>> {
    /// The callback to call, takes state and returns the needed data for the reactive
    /// transformation
    callback: Box<dyn Fn(&mut RenderCtx<C>) -> K>,
    /// The node to apply transformations to
    node: web_sys::Element,
    /// Vector of various objects to be kept alive for the duration of the rendered content
    keep_alive: Vec<KeepAlive>,
    /// Hooks to use
    hooks: Vec<HookKey>,
    /// The state needed to apply the transformation
    state: K::State,
}

impl<C: Component, K: ReactiveValue<C>> ReactiveHook<C> for SimpleReactive<C, K> {
    fn drop_us(self: Box<Self>) -> Vec<HookKey> {
        self.hooks
    }

    fn update(&mut self, ctx: &mut State<C>, you: HookKey) -> UpdateResult {
        let hooks = std::mem::take(&mut self.hooks);

        ctx.clear();
        self.keep_alive.clear();
        let value = (self.callback)(&mut RenderCtx {
            ctx,
            render_state: RenderingState {
                keep_alive: &mut self.keep_alive,
                hooks: &mut self.hooks,
                parent_dep: you,
            },
        });
        ctx.reg_dep(you);

        value.apply(
            ctx,
            &mut RenderingState {
                keep_alive: &mut self.keep_alive,
                hooks: &mut self.hooks,
                parent_dep: you,
            },
            &self.node,
            &mut self.state,
        );
        UpdateResult::DropHooks(hooks)
    }
}

impl<C: Component, K: ReactiveValue<C> + 'static> SimpleReactive<C, K> {
    /// Creates a new simple reactive hook, applying the initial transformation.
    /// Returns a Rc of the hook
    pub(crate) fn init_new(
        callback: Box<dyn Fn(&mut RenderCtx<C>) -> K>,
        node: web_sys::Element,
        ctx: &mut State<C>,
    ) -> HookKey {
        let me = ctx.insert_hook(Box::new(DummyHook));

        let mut this = Self {
            callback,
            node,
            keep_alive: Vec::new(),
            hooks: Vec::new(),
            state: K::State::default(),
        };
        this.update(ctx, me);

        ctx.set_hook(me, Box::new(this));

        me
    }
}

/// Reactivly set a element attribute
pub(crate) struct ReactiveAttribute<T> {
    /// The attribute name to set
    pub(crate) name: &'static str,
    /// The attribute value to apply
    pub(crate) data: T,
}

impl<C: Component, T: ToAttribute<C>> ReactiveValue<C> for ReactiveAttribute<T> {
    type State = ();

    fn apply(
        self,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
        node: &web_sys::Element,
        _state: &mut Self::State,
    ) {
        match self.data.calc_attribute(self.name, node) {
            AttributeResult::SetIt(Some(res)) => {
                debug_expect!(
                    node.set_attribute(self.name, &res),
                    "Failed to update attribute"
                );
            }
            AttributeResult::SetIt(None) => {
                debug_expect!(
                    node.remove_attribute(self.name),
                    "Failed to remove attribute"
                );
            }
            AttributeResult::IsDynamic(apply) => apply(ctx, render_state),
        }
    }
}

/// Reactively set a element class
pub(crate) struct ReactiveClass<T> {
    /// The class value to apply
    pub(crate) data: T,
}

impl<C: Component, T: ToClass<C>> ReactiveValue<C> for ReactiveClass<T> {
    type State = Option<Cow<'static, str>>;

    fn apply(
        self,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
        node: &web_sys::Element,
        state: &mut Self::State,
    ) {
        let class_list = node.class_list();

        let new = match self.data.calc_class(node) {
            ClassResult::SetIt(res) => {
                match (&state, &res) {
                    (None, None) => {}
                    (Some(prev), None) => {
                        debug_expect!(class_list.remove_1(prev), "Failed to remove class");
                    }
                    (None, Some(new)) => {
                        debug_expect!(class_list.add_1(new), "Failed to add class");
                    }
                    (Some(prev), Some(new)) => {
                        debug_expect!(class_list.replace(prev, new), "Failed to replace class");
                    }
                }

                res
            }
            ClassResult::Dynamic(dynamic) => {
                log::debug!("Nested reactive classes, doing naive instant remove.");
                if let Some(prev) = state.take() {
                    debug_expect!(
                        class_list.remove_1(&prev),
                        "Failed to remove class from element"
                    );
                }

                dynamic(ctx, render_state);
                None
            }
        };
        *state = new;
    }
}
