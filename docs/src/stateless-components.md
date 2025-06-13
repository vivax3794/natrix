# State-Less Components

Stateless components are, technically speaking, not even a explicit feature of natrix. But just a by product of the design. They are simply functions (or methods, or anything else) that returns `impl Element<...>`, this chapther is here to outline what they usually look like and some common patterns. And is not a exhustive list of whats possible with the amazing work of art thats rust trait system.

As mentioned in the component chaphter `Element` is generic over the component state it references, this is true even if it doesnt reference any state.
This means that your stateless component functions needs to be generic:

```rust
# extern crate natrix;
# use natrix::prelude::*;

fn hello<C: Component>() -> impl Element<C> {
    e::h1().text("Hello World")
}
```

These can then be called from within a component, or even from other stateless components:

```rust
# extern crate natrix;
# use natrix::prelude::*;
#
# fn hello<C: Component>() -> impl Element<C> {
#     e::h1().text("Hello World")
# }
# #[derive(Component)]
# struct HelloWorld;
impl Component for HelloWorld {
    fn render() -> impl Element<Self> {
        e::div()
            .text("Hello World")
            .child(hello())
    }
}
```

## Passing arguments

Since they are just functions stateless functions can take arguments like any other function.

```rust
# extern crate natrix;
# use natrix::prelude::*;
fn hello<C: Component>(name: String) -> impl Element<C> {
    e::h1().text("Hello ").text(name)
}
```

Now this has a downside, imagine if we wanted `name` to be reactive? we would have to put the entire component call in the reactive closure.

```rust
# extern crate natrix;
# use natrix::prelude::*;
#
# fn hello<C: Component>(name: String) -> impl Element<C> {
#     e::h1().text("Hello ").text(name.clone())
# }
#[derive(Component)]
struct HelloWorld {
    name: String,
}

impl Component for HelloWorld {
    fn render() -> impl Element<Self> {
        e::div()
            .text("Hello World")
            .child(|ctx: R<Self>| hello(ctx.name.clone()))
    }
}
```

But what if `hello` was actually really complex? We would be recreating that entire dom tree every time `name` changes. The solution is to actually make `hello` generic over the hello argument!

```rust
# extern crate natrix;
# use natrix::prelude::*;
fn hello<C: Component>(name: impl Element<C>) -> impl Element<C> {
    e::h1().text("Hello ").child(name)
}
```

Now we can make just the name part reactive

```rust
# extern crate natrix;
# use natrix::prelude::*;
#
# fn hello<C: Component>(name: impl Element<C>) -> impl Element<C> {
#     e::h1().text("Hello ").text(name)
# }
#[derive(Component)]
struct HelloWorld {
    name: String,
}

impl Component for HelloWorld {
    fn render() -> impl Element<Self> {
        e::div()
            .text("Hello World")
            .child(hello(|ctx: R<Self>| ctx.name.clone()))
    }
}
```

And now only the name part will be recreated when `name` changes.

## Events

Natrix provides the [`EventHandler`](dom::events::EventHandler) trait which makes taking event handlers in stateless components easier, in reality this trait is only implement for closures of the appropriate signature.

```rust
# extern crate natrix;
# use natrix::prelude::*;
use natrix::dom::EventHandler;

fn fancy_button<C: Component>(
    on_click: impl EventHandler<C, events::Click>,
) -> impl Element<C> {
    e::button()
        .text("Click me!")
        .on(on_click)
}
```

This can be used like this:

```rust
# extern crate natrix;
# use natrix::prelude::*;
# use natrix::dom::EventHandler;
# fn fancy_button<C: Component>(
#     on_click: impl EventHandler<C, events::Click>,
# ) -> impl Element<C> {
#     e::button()
#         .text("Click me!")
#         .on(on_click)
# }
#
#[derive(Component)]
struct HelloWorld {
    counter: u8,
};

impl Component for HelloWorld {
    fn render() -> impl Element<Self> {
        e::div()
            .child(fancy_button(|ctx: E<Self>, _, _| {
                *ctx.counter += 1;
            }))
    }
}
```

----

## Concrete stateless components

Yet again this isnt a explicit feature, but rather "common sense".
You can also write helper functions that dont use generics, and hence can declare their closures directly.

```rust
# extern crate natrix;
# use natrix::prelude::*;
#[derive(Component)]
struct Counter {
    value: i16,
}

fn change_button(delta: i16) -> impl Element<Counter> {
    e::button()
        .text(delta)
        .on::<events::Click>(move |ctx: E<Counter>, _, _| {
            *ctx.value += delta;
        })
}

impl Component for Counter {
    fn render() -> impl Element<Self> {
        e::div()
            .child(change_button(-10))
            .child(change_button(-1))
            .text(|ctx: R<Self>| *ctx.value)
            .child(change_button(1))
            .child(change_button(10))
    }
}
```

## Non event handling closures.

Also nothing stopping you from taking explicit closures.

```rust
# extern crate natrix;
# use natrix::prelude::*;
fn change_button<C: Component>(
    delta: i16,
    modify: impl Fn(E<C>, i16) + 'static,
) -> impl Element<C> {
    e::button()
        .text(delta)
        .on::<events::Click>(move |ctx: E<C>, _, _| modify(ctx, delta))
}

fn change_buttons<C: Component>(
    modify: impl Fn(E<C>, i16) + Clone + 'static
) -> impl Element<C> {
    e::div()
        .child(change_button(-10, modify.clone()))
        .child(change_button(-1, modify.clone()))
        .child(change_button(1, modify.clone()))
        .child(change_button(10, modify.clone()))
}

#[derive(Component)]
struct Counter {
    value: i16,
}

impl Component for Counter {
    fn render() -> impl Element<Self> {
        e::div()
            .text(|ctx: R<Self>| *ctx.value)
            .child(change_buttons(|ctx: E<Self>, delta| {*ctx.value += delta;}))
    }
}
```
