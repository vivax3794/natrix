use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;

use crate::element::SealedElement;
use crate::get_document;
use crate::html_elements::ToAttribute;
use crate::signal::{RcDep, RcDepWeak, ReactiveHook, RenderingState};
use crate::state::{ComponentData, KeepAlive, State};
use crate::utils::{RcCmpPtr, WeakCmpPtr};

struct DummyHook;
impl<C: ComponentData> ReactiveHook<C> for DummyHook {
    fn update(&mut self, _ctx: &mut State<C>, _you: RcDepWeak<C>) {}
}

pub(crate) struct ReactiveNode<C, E> {
    callback: Box<dyn Fn(&State<C>) -> E>,
    target_node: web_sys::Node,
    keep_alive: Vec<KeepAlive>,
}

impl<C: ComponentData, E: SealedElement<C>> ReactiveNode<C, E> {
    fn render_inplace(&mut self, ctx: &mut State<C>, you: RcDepWeak<C>) {
        let new_node = self.render(ctx, you);

        let parent = self.target_node.parent_node().expect("No parent found");
        parent
            .replace_child(&new_node, &self.target_node)
            .expect("Failed to replace node");
        self.target_node = new_node;
    }

    fn render(
        &mut self,
        ctx: &mut State<C>,
        you: crate::utils::WeakCmpPtr<RefCell<Box<dyn ReactiveHook<C>>>>,
    ) -> web_sys::Node {
        ctx.clear();
        let element = (self.callback)(ctx);
        ctx.reg_dep(you);

        self.keep_alive.clear();
        let mut state = RenderingState {
            keep_alive: &mut self.keep_alive,
        };
        element.render(ctx, &mut state)
    }

    pub(crate) fn create_inital(
        callback: Box<dyn Fn(&State<C>) -> E>,
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

        let node = this.render(ctx, WeakCmpPtr(result_weak));
        this.target_node = node.clone();

        *result_owned.0.borrow_mut() = Box::new(this);

        (result_owned, node)
    }
}

pub(crate) trait ReactiveValue<C> {
    fn apply(self, ctx: &mut State<C>, render_state: &mut RenderingState, node: &web_sys::Element);
}

pub(crate) struct SimpleReactive<C, K> {
    callback: Box<dyn Fn(&State<C>) -> K>,
    node: web_sys::Element,
    keep_alive: Vec<KeepAlive>,
}

impl<C: ComponentData, K: ReactiveValue<C>> ReactiveHook<C> for SimpleReactive<C, K> {
    fn update(&mut self, ctx: &mut State<C>, you: RcDepWeak<C>) {
        ctx.clear();
        let value = (self.callback)(ctx);
        ctx.reg_dep(you);

        self.keep_alive.clear();
        let mut state = RenderingState {
            keep_alive: &mut self.keep_alive,
        };
        value.apply(ctx, &mut state, &self.node);
    }
}

impl<C: ComponentData, K: ReactiveValue<C> + 'static> SimpleReactive<C, K> {
    pub(crate) fn init_new(
        callback: Box<dyn Fn(&State<C>) -> K>,
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
        this.update(ctx, result_weak);

        *result.0.borrow_mut() = Box::new(this);

        result
    }
}

pub(crate) struct ReactiveAttribute<T> {
    pub(crate) name: &'static str,
    pub(crate) data: T,
}

impl<C, T: ToAttribute<C>> ReactiveValue<C> for ReactiveAttribute<T> {
    fn apply(self, ctx: &mut State<C>, render_state: &mut RenderingState, node: &web_sys::Element) {
        Box::new(self.data).apply_attribute(self.name, node, ctx, render_state);
    }
}
