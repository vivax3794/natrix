# State-Less Components

Stateless components are, technically speaking, not even a explicit feature of natrix. But just a by product of the design. They are simply functions (or methods, or anything else) that returns `impl Element<...>`, this chapther is here to outline what they usually look like and some common patterns. 

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
``````

And now only the name part will be recreated when `name` changes.

## Events
Natrix provides the [`EventHandler`](callbacks::EventHandler) trait which makes taking event handlers in stateless components easier, in reality this trait is only implement for closures of the appropriate signature.

```rust
# extern crate natrix;
# use natrix::prelude::*;
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
            .child(fancy_button(|ctx: E<Self>, _| {
                *ctx.counter += 1;
            }))
    }
}
```
