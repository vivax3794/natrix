# Components

[`Component`](component::Component)s are important part of natrix, and are the core of the [reactivity system](reactivity.md). 

> [!NOTE]
> If you are looking for a way to create a component without any state there is a more light weight alternative in the [Stateless Components](stateless-components.md) section.

## Basic Components
Components are implemented by using the `Component` derive macro **and** manually implementing the [`Component`](component::Component) trait. This is because the derive macro actually implements the [`ComponentBase`](component::ComponentBase) trait.

Components have 3 required items, [`render`](component::Component::render), [`EmitMessage`](component::Component::EmitMessage) and [`ReceiveMessage`](component::Component::ReceiveMessage).

```rust,no_run
# extern crate natrix;
# use natrix::prelude::*;
#
#[derive(Component)]
struct HelloWorld;

impl Component for HelloWorld {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::div().text("Hello World")
    }
}
#
# fn main() {
#     mount(HelloWorld);
# }
```

> [!IMPORTANT]
> With the [`nightly`](features.md#nightly) feature you can omit the `EmitMessage` and `ReceiveMessage` types, as they default to [`NoMessages`](component::NoMessages). In all other examples we will omit them for simplicity as nightly is the recommended toolchain.

The `render` function should return a type that implements the [`Element`](element::Element) trait. This is usually done by using the [Html Elements](html.md) or rust types that implement the [`Element`](element::Element) trait. Elements are generic over the component type, hence `impl Element<Self>`, this provides strong type guarantees for reactivity and event handlers without needing to capture signals like in other frameworks.

In contrast to frameworks like React, the `render` function is not called every time the component needs to be updated. Instead, it is only called when the component is mounted. This is because natrix uses a [reactivity system](reactivity.md) that allows fine-grained reactivity.
## State

Now components with no state are not very useful (well they are, but you should use [Stateless Components](stateless-components.md) instead), so lets add some state to our component. This is done simply by adding fields to the struct.

```rust,no_run
# extern crate natrix;
# use natrix::prelude::*;
#
#[derive(Component)]
struct HelloWorld {
    counter: u8,
}

impl Component for HelloWorld {
    fn render() -> impl Element<Self> {
        e::button()
    }
}

fn main() {
    mount(HelloWorld { counter: 0 });
}
```

As you can see when mounting a component with state you simply construct the instance without needing any wrappers.

### Displaying State

> [!NOTE]
> The [`.text`](html_elements::HtmlElement::text) method is a alias for [`.child`](html_elements::HtmlElement::child), so the following section applies to both.

Natrix uses callbacks similar to other frameworks, but instead of capturing signals callbacks instead take a reference to the component. This is mainly done via the [`R`](state::R) type alias, `R<Self>` is a alias for `&mut RenderCtx<Self>`

```rust,no_run
# extern crate natrix;
# use natrix::prelude::*;
#
# #[derive(Component)]
# struct HelloWorld {
#     counter: u8,
# }
impl Component for HelloWorld {
    fn render() -> impl Element<Self> {
        e::button()
            .text(|ctx: R<Self>| *ctx.counter)
    }
}
#
# fn main() {
#     mount(HelloWorld { counter: 0 });
# }
```
We need to specify the argument type of the closure, this is because of limitation in the type inference system. The closures also can return anything that implements the [`Element`](element::Element) trait, so you can use any of the [`Html Elements`](html.md) or any other type that implements the [`Element`](element::Element) trait.

> [!TIP]
> See the [reactivity](reactivity.md) section for more information on how fine grained reactivity works and best practices.

### Updating State

Updating state is done very similarly, but using [`E`](state::E), the [`.on`](html_elements::HtmlElement::on) method takes a callback that is called when the event is triggered. The callback takes a reference to the component and the event as arguments. The event is passed as a generic type, so you can use any event that implements the [`Event`](events::Event) trait. the second argument will automatically be inferred to the type of the event. for example the [`Click`](events::Click) event will be passed as a [`MouseEvent`](web_sys::MouseEvent) type.

```rust,no_run
# extern crate natrix;
# use natrix::prelude::*;
#
# #[derive(Component)]
# struct HelloWorld {
#     counter: u8,
# }
impl Component for HelloWorld {
    fn render() -> impl Element<Self> {
        e::button()
            .text(|ctx: R<Self>| *ctx.counter)
            .on::<events::Click>(|ctx: E<Self>, _| {
                *ctx.counter += 1;
            })
    }
}
#
# fn main() {
#     mount(HelloWorld { counter: 0 });
# }
```
