//! Implements the reactive hooks for updating the dom in response to signal changessz.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;

use crate::element::Element;
use crate::get_document;
use crate::html_elements::ToAttribute;
use crate::signal::{RcDep, RcDepWeak, ReactiveHook, RenderingState};
use crate::state::{ComponentData, KeepAlive, RenderCtx, State};
use crate::utils::{RcCmpPtr, WeakCmpPtr};

/// A noop hook used to fill the `Rc<RefCell<...>>` while the inital render pass runs so that that
/// a real hook can be swapped in once initalized
struct DummyHook;
impl<C: ComponentData> ReactiveHook<C> for DummyHook {
    fn update(&mut self, _ctx: &mut State<C>, _you: &RcDepWeak<C>) {}
    fn drop_children_early(&mut self) {}
}

/// Reactive hook for swapping out a entire dom node.
pub(crate) struct ReactiveNode<C, E> {
    /// The callback to produce nodes
    callback: Box<dyn Fn(RenderCtx<C>) -> E>,
    /// The current renderd node to replace
    target_node: web_sys::Node,
    /// Vector of various objects to be kept alive for the duration of the renderd content
    keep_alive: Vec<KeepAlive>,
}

impl<C: ComponentData, E: Element<C>> ReactiveNode<C, E> {
    /// Render this hook and simply return the node
    ///
    /// IMPORTANT: This function works with the assumption what it returns will be put in its
    /// `target_node` field. This function is split out to facilitate `Self::create_inital`
    fn render(&mut self, ctx: &mut State<C>, you: &RcDepWeak<C>) -> web_sys::Node {
        ctx.clear();
        let element = (self.callback)(RenderCtx(ctx));
        ctx.reg_dep(you);

        let mut state = RenderingState {
            keep_alive: &mut self.keep_alive,
        };
        element.render(ctx, &mut state)
    }

    /// Create a new `ReactiveNode` registering the inital depdencies and returning both the `Rc`
    /// reference to it and the inital node (Which should be inserted in the dom)
    pub(crate) fn create_inital(
        callback: Box<dyn Fn(RenderCtx<C>) -> E>,
        ctx: &mut State<C>,
    ) -> (RcDep<C>, web_sys::Node) {
        let dummy_node = get_document()
            .body()
            .expect("WHAT?")
            .dyn_into()
            .expect("HUH?!");

        let result_owned: RcDep<C> = RcCmpPtr(Rc::new(RefCell::new(Box::new(DummyHook))));
        let result_weak = Rc::downgrade(&result_owned.0);

        let mut this = Self {
            callback,
            target_node: dummy_node,
            keep_alive: Vec::new(),
        };

        let node = this.render(ctx, &WeakCmpPtr(result_weak));
        this.target_node = node.clone();

        *result_owned.0.borrow_mut() = Box::new(this);

        (result_owned, node)
    }
}

impl<C: ComponentData, E: Element<C>> ReactiveHook<C> for ReactiveNode<C, E> {
    fn update(&mut self, ctx: &mut State<C>, you: &RcDepWeak<C>) {
        let this = &mut *self;
        let new_node = this.render(ctx, you);

        let parent = this.target_node.parent_node().expect("No parent found");
        parent
            .replace_child(&new_node, &this.target_node)
            .expect("Failed to replace node");
        this.target_node = new_node;
    }

    fn drop_children_early(&mut self) {
        self.keep_alive.clear();
    }
}

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
    callback: Box<dyn Fn(RenderCtx<C>) -> K>,
    /// The node to apply transformations to
    node: web_sys::Element,
    /// Vector of various objects to be kept alive for the duration of the renderd content
    keep_alive: Vec<KeepAlive>,
}

impl<C: ComponentData, K: ReactiveValue<C>> ReactiveHook<C> for SimpleReactive<C, K> {
    fn update(&mut self, ctx: &mut State<C>, you: &RcDepWeak<C>) {
        ctx.clear();
        let value = (self.callback)(RenderCtx(ctx));
        ctx.reg_dep(you);

        let mut state = RenderingState {
            keep_alive: &mut self.keep_alive,
        };
        value.apply(ctx, &mut state, &self.node);
    }
    fn drop_children_early(&mut self) {
        self.keep_alive.clear();
    }
}

impl<C: ComponentData, K: ReactiveValue<C> + 'static> SimpleReactive<C, K> {
    /// Creates a new simple reactive hook, applying the inital transformation.
    /// Returns a Rc of the hook
    pub(crate) fn init_new(
        callback: Box<dyn Fn(RenderCtx<C>) -> K>,
        node: web_sys::Element,
        ctx: &mut State<C>,
    ) -> RcDep<C> {
        let result: RcDep<C> = RcCmpPtr(Rc::new(RefCell::new(Box::new(DummyHook))));
        let result_weak = WeakCmpPtr(Rc::downgrade(&result.0));

        let mut this = Self {
            callback,
            node,
            keep_alive: Vec::new(),
        };
        this.update(ctx, &result_weak);

        *result.0.borrow_mut() = Box::new(this);

        result
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
