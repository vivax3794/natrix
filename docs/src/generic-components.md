# Generic Components

Components can be generic just the way you would expect.

```rust
# extern crate natrix;
# use natrix::prelude::*;
#
#[derive(Component)]
struct MyComponent<T>(T, T);

impl<T: Eq + 'static> Component for MyComponent<T> {
    fn render() -> impl Element<Self> {
        e::div()
            .text(|ctx: R<Self>| {
                if *ctx.0 == *ctx.1 {
                    "Equal"
                } else {
                    "Not Equal"
                }
            })
    }
}
```

## Generic over [`Element`](element::Element)/[`ToAttribute`](html_elements::ToAttribute)

If you want to be generic over something with a [`Element`](element::Element) bound you will run into a recursion error in the type checker.

```rust,compile_fail
# extern crate natrix;
# use natrix::prelude::*;
#
#[derive(Component)]
struct MyComponent<T>(T);

impl<T: Element<Self> + Clone> Component for MyComponent<T> {
    fn render() -> impl Element<Self> {
        e::div()
            .child(|ctx: R<Self>| ctx.0.clone())
    }
}
#
# fn main() {
#     mount(MyComponent("Hello World".to_string()));
# }
```

The problem here is that [`Element`](element::Element) needs to be generic over the component, so `Element<Self>`,
but its also enforces a [`Component`](component::Component) bound on its generic, this means that in order to prove `MyComponent<T>` implements [`Component`](component::Component) it must first prove `MyComponent<T>` implements [`Component`](component::Component), which rust doesnt like and errors out on. To solve this you can use the [`NonReactive`](component::NonReactive) wrapper which will allow you to use `Element<()>` as the generic bound. As the name implies this essentially means that part of the dom tree cant be reactive.

[`NonReactive`](component::NonReactive) is essentially a wrapper that swaps out the component instance its given with `()`.

```rust,no_run
# extern crate natrix;
use natrix::prelude::*;
use natrix::component::NonReactive;

#[derive(Component)]
struct MyComponent<T>(T);

impl<T: Element<()> + Clone> Component for MyComponent<T> {
    fn render() -> impl Element<Self> {
        e::div()
            .child(|ctx: R<Self>| NonReactive(ctx.0.clone()))
    }
}
#
# fn main() {
#     mount(MyComponent("Hello World".to_string()));
# }
```

[`NonReactive`](component::NonReactive) also implements [`ToAttribute`](html_elements::ToAttribute) so a similar trick can be used for it.
