# Reactivity

## Callbacks

You have already seen `|ctx: &mut RenderCtx<Self>| ...` used in the varying examples in the book.
Lets go into some more detail about what this does.

The callbacks return a value that implements [`Element`](dom::element::Element), internally the framework will register which fields you accessed.
And when those fields change, the framework will recall the callback and update the element with the result.

Natrix does **_not_** use a virtual dom, meaning when a callback is re-called the framework will swap out the entire associated element.

> [!IMPORTANT]
> Natrix assumes render callbacks are pure, meaning they do not have side effects.
> If you use stuff like interior mutability it will break framework assumptions,
> and will likely lead to panics or desynced state.

### Execution Guarantees

Natrix _only_ makes the following guarantees about when a callback will be called:

- It will not be called if a parent is dirty.

Thats it, natrix does not make any guarantees about the order of sibling callbacks.

## Returning different kinds of elements.
Sometimes two branches returns different kinds of elements, this can be solved using `Result`, or by pre-rendering them using [`.render`](dom::element::Element::render). Which produces the internal result of a element render (which itself implements `Element` for this exact purpose)

```rust
# extern crate natrix;
# use natrix::prelude::*;
# #[derive(State)]
# struct HelloWorld {
#     counter: Signal<u8>,
# }
# fn render_hello_world() -> impl Element<HelloWorld> {
e::div()
    .child(|ctx: &mut RenderCtx<HelloWorld>| {
        if *ctx.counter > 10 {
            e::h1().text("Such big").render()
        } else {
            "Oh no such small".render()
        }
    })
# }

```

> [!TIP]
> For handling multiple types of html elements, theres [`.generic()`](dom::html_elements::HtmlElement::generic), which returns `HtmlElement<C, ()>`, i.e erases the dom tag, allowing you to for example construct different tags in a `if`, and then later call methods on it. Ofc doing this means only global attribute helpers can be used, but you can always use `.attr` directly.

## Signal-based Reactivity

Natrix uses `Signal<T>` types to track reactive state. When you access a signal in a render callback, the framework automatically tracks the dependency and will re-run the callback when the signal changes.

```rust
# extern crate natrix;
# use natrix::prelude::*;
#[derive(State)]
struct Counter {
    value: Signal<i32>,
}

fn render_counter() -> impl Element<Counter> {
    e::div()
        .child(e::button()
            .text(|ctx: &mut RenderCtx<Counter>| *ctx.value)
            .on::<events::Click>(|ctx: &mut Ctx<Counter>, _, _| {
                *ctx.value += 1;
            }))
}
```

The reactivity system automatically tracks when `ctx.value` is accessed and will re-run the text callback whenever the value changes.