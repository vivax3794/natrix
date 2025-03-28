//! Implements the reactive hooks for updating the dom in response to signal changessz.

use crate::element::Element;
use crate::html_elements::ToAttribute;
use crate::signal::{ReactiveHook, RenderingState, UpdateResult};
use crate::state::{ComponentData, HookKey, KeepAlive, RenderCtx, State};
use crate::utils::debug_expect;
use crate::{get_document, type_macros};

/// A noop hook used to fill the `Rc<RefCell<...>>` while the inital render pass runs so that that
/// a real hook can be swapped in once initalized
pub(crate) struct DummyHook;
impl<C: ComponentData> ReactiveHook<C> for DummyHook {
    fn update(&mut self, _ctx: &mut State<C>, _you: HookKey) -> UpdateResult {
        UpdateResult::Nothing
    }
}

/// Reactive hook for swapping out a entire dom node.
pub(crate) struct ReactiveNode<C, E> {
    /// The callback to produce nodes
    callback: Box<dyn Fn(&mut RenderCtx<C>) -> E>,
    /// The current renderd node to replace
    target_node: web_sys::Node,
    /// Vector of various objects to be kept alive for the duration of the renderd content
    keep_alive: Vec<KeepAlive>,
    /// Hooks that are a child of this
    hooks: Vec<HookKey>,
}

impl<C: ComponentData, E: Element<C>> ReactiveNode<C, E> {
    /// Render this hook and simply return the node
    ///
    /// IMPORTANT: This function works with the assumption what it returns will be put in its
    /// `target_node` field. This function is split out to facilitate `Self::create_inital`
    fn render(&mut self, ctx: &mut State<C>, you: HookKey) -> web_sys::Node {
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

    /// Create a new `ReactiveNode` registering the inital depdencies and returning both the `Rc`
    /// reference to it and the inital node (Which should be inserted in the dom)
    pub(crate) fn create_inital(
        callback: Box<dyn Fn(&mut RenderCtx<C>) -> E>,
        ctx: &mut State<C>,
    ) -> (HookKey, web_sys::Node) {
        #[expect(
            clippy::expect_used,
            reason = "If there is no body, then wtf is our job."
        )]
        let dummy_node: web_sys::Node = get_document().body().expect("<body> not found").into();

        let me = ctx.insert_hook(Box::new(DummyHook));

        let mut this = Self {
            callback,
            target_node: dummy_node,
            keep_alive: Vec::new(),
            hooks: Vec::new(),
        };
        let node = this.render(ctx, me);
        this.target_node = node.clone();
        ctx.set_hook(me, Box::new(this));

        (me, node)
    }

    /// Pulled out update method to facilite marking it as `default` on nightly
    fn update(&mut self, ctx: &mut State<C>, you: HookKey) {
        let new_node = self.render(ctx, you);

        let Some(parent) = self.target_node.parent_node() else {
            debug_assert!(false, "Parent node of target node not found.");
            return;
        };

        debug_expect!(
            parent.replace_child(&new_node, &self.target_node),
            "Failed to replace parent"
        );
        self.target_node = new_node;
    }
}

impl<C: ComponentData, E: Element<C>> ReactiveHook<C> for ReactiveNode<C, E> {
    #[cfg(not(nightly))]
    fn update(&mut self, ctx: &mut State<C>, you: HookKey) -> UpdateResult {
        self.update(ctx, you);
        UpdateResult::Nothing
    }

    #[cfg(nightly)]
    default fn update(&mut self, ctx: &mut State<C>, you: HookKey) -> UpdateResult {
        self.update(ctx, you);
        UpdateResult::Nothing
    }

    fn drop_deps(&mut self) -> Option<std::vec::Drain<'_, HookKey>> {
        self.keep_alive.clear();
        Some(self.hooks.drain(..))
    }
}

#[cfg(nightly)]
impl<C: ComponentData> ReactiveHook<C> for ReactiveNode<C, String> {
    fn update(&mut self, ctx: &mut State<C>, you: HookKey) -> UpdateResult {
        use wasm_bindgen::JsCast;

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

        if let Some(target_node) = self.target_node.dyn_ref::<web_sys::Text>() {
            target_node.set_text_content(Some(&element));
        } else {
            debug_assert!(false, "`String` Node wasnt a text node");
        }

        UpdateResult::Nothing
    }
}

/// generate nightly only optimization for ints and floats
macro_rules! node_specialize_int {
    ($type:ty, $fmt:ident) => {
        #[cfg(nightly)]
        impl<C: ComponentData> ReactiveHook<C> for ReactiveNode<C, $type> {
            fn update(&mut self, ctx: &mut State<C>, you: HookKey) -> UpdateResult {
                use wasm_bindgen::JsCast;

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

                let mut buffer = $fmt::Buffer::new();
                let result = buffer.format(element);

                if let Some(target_node) = self.target_node.dyn_ref::<web_sys::Text>() {
                    target_node.set_text_content(Some(result));
                } else {
                    debug_assert!(false, "Numeric Node wasnt a text node");
                }

                UpdateResult::Nothing
            }
        }
    };
}

type_macros::numerics!(node_specialize_int);

/// A trait to allow `SimpleReactive` to deduplicate common reactive logic for attributes, classes,
/// styles, etc
pub(crate) trait ReactiveValue<C> {
    /// Actually apply the change
    fn apply(self, ctx: &mut State<C>, render_state: &mut RenderingState, node: &web_sys::Element);
}

/// A common wrapper for simple reactive operations to deduplicate depdency tracking code
pub(crate) struct SimpleReactive<C, K> {
    /// The callback to call, takes state and returns the needed data for the reactive
    /// transformation
    callback: Box<dyn Fn(&mut RenderCtx<C>) -> K>,
    /// The node to apply transformations to
    node: web_sys::Element,
    /// Vector of various objects to be kept alive for the duration of the renderd content
    keep_alive: Vec<KeepAlive>,
    /// Hooks to use
    hooks: Vec<HookKey>,
}

impl<C: ComponentData, K: ReactiveValue<C>> ReactiveHook<C> for SimpleReactive<C, K> {
    fn update(&mut self, ctx: &mut State<C>, you: HookKey) -> UpdateResult {
        ctx.clear();
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
        );
        UpdateResult::Nothing
    }

    fn drop_deps(&mut self) -> Option<std::vec::Drain<'_, HookKey>> {
        self.keep_alive.clear();
        Some(self.hooks.drain(..))
    }
}

impl<C: ComponentData, K: ReactiveValue<C> + 'static> SimpleReactive<C, K> {
    /// Creates a new simple reactive hook, applying the inital transformation.
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

impl<C, T: ToAttribute<C>> ReactiveValue<C> for ReactiveAttribute<T> {
    fn apply(self, ctx: &mut State<C>, render_state: &mut RenderingState, node: &web_sys::Element) {
        Box::new(self.data).apply_attribute(self.name, node, ctx, render_state);
    }
}
