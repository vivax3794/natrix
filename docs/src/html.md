# Html

Html elements are the building blocks of web pages. While other rust frameworks aim for a JSX-like syntax, this library uses a more traditional approach.
The goal is to provide a simple and efficient way to create HTML elements without the need for complex syntax, we use the idomatic rust builder pattern.

Natrix uses a single [`HtmlElement`](dom::html_elements::HtmlElement) struct to represent all HTML elements. But exposes helper functions for each tag.
These are found along side the `HtmlElement` struct in the [`html_elements`](dom::html_elements) module.
Which will most commonly be used via the `e` alias in the [`prelude`](prelude) module.

```rust,no_run
# extern crate natrix;
# use natrix::prelude::*;
# let _: e::HtmlElement<(), _> =
e::div()
# ;
```

If you need to construct a element with a tag not found in the library you can use [`HtmlElement::new`](dom::html_elements::HtmlElement::new).

```rust,no_run
# extern crate natrix;
# use natrix::prelude::*;
# let _: e::HtmlElement<(), ()> =
e::HtmlElement::new("custom_tag")
# ;
```

## Children

Children are added using the [`.child`](dom::html_elements::HtmlElement::child) method. This method takes a single child element and adds it to the parent element.

```rust,no_run
# extern crate natrix;
# use natrix::prelude::*;
# let _: e::HtmlElement<(), _> =
e::div()
    .child(e::button())
    .child(e::h1().child("Hello World!"))
# ;
```

> [!TIP]
> the [`.text`](dom::html_elements::HtmlElement::text) method is a alias for [`.child`](dom::html_elements::HtmlElement::child)

Child elements can be any type that implements the [`Element`](dom::element::Element) trait, including other [`HtmlElement`](dom::html_elements::HtmlElement) instances, and stdlib types like [`String`](std::string::String), [`&str`](std::primitive::str), [`i32`](std::primitive::i32), as well as containers such as [`Option`](std::option::Option) and [`Result`](std::result::Result).

Child elements can also be reactive as closures implement the [`Element`](dom::element::Element) trait.

```rust
# extern crate natrix;
# use natrix::prelude::*;
#
# #[derive(State)]
# struct MyComponent {
#     pub is_active: Signal<bool>,
# }
#
# fn render(ctx: &mut RenderCtx<MyComponent>) -> impl Element<MyComponent> {
e::div()
    .child(e::button()
        .text("Click me!")
        .on::<events::Click>(|ctx: &mut Ctx<MyComponent>, _, _| {
            *ctx.is_active.write() = !*ctx.is_active.read();
        })
    )
    .child(|ctx: &mut RenderCtx<MyComponent>| {
        if *ctx.is_active.read() {
            Some(e::p().text("Active!"))
        } else {
            None
        }
    })
# }
```

## `format_elements`
You can use the [`format_elements`](format_elements) macro to get `format!` like ergonomics for elements.
```rust
# extern crate natrix;
# use natrix::prelude::*;
#
# #[derive(State)]
# struct MyComponent {
#     pub counter: Signal<u8>,
#     pub target: Signal<u8>,
# }
#
# fn render(ctx: &mut RenderCtx<MyComponent>) -> impl Element<MyComponent> {
e::h1().children(
    natrix::format_elements!(|ctx: &mut RenderCtx<MyComponent>| "Counter is {}, just {} clicks left!", *ctx.counter.read(), *ctx.target.read() - *ctx.counter.read())
)
# }
```
Which expands to effectively:
```rust
# extern crate natrix;
# use natrix::prelude::*;
#
# #[derive(State)]
# struct MyComponent {
#     pub counter: Signal<u8>,
#     pub target: Signal<u8>,
# }
#
# fn render(ctx: &mut RenderCtx<MyComponent>) -> impl Element<MyComponent> {
e::h1()
    .text("Counter is ")
    .child(|ctx: &mut RenderCtx<MyComponent>| *ctx.counter.read())
    .text(", just ")
    .child(|ctx: &mut RenderCtx<MyComponent>| *ctx.target.read() - *ctx.counter.read())
    .text(" clicks left!")
# }
```

I.e this is much more performant than `format!` for multiple reasons:
* You avoid the format machinery overhead.
* You get fine-grained reactivty for specific parts of the text.


## Attributes

Attributes are set using the [`.attr`](dom::html_elements::HtmlElement::attr) method. This method takes a key and a value, and sets the attribute on the element.

```rust,no_run
# extern crate natrix;
# use natrix::prelude::*;
# let _: e::HtmlElement<(), _> =
e::div()
    .attr("data-foo", "bar")
    .attr("data-baz", "qux")
# ;
```

Most standard html attributes have type-safe helper functions, for example `id`, `class`, `href`, `src`, etc.
For non-global attributes natrix only exposes them on the supporting elements.

```rust,no_run
# extern crate natrix;
# use natrix::prelude::*;
use natrix::dom::attributes;

# let _: e::HtmlElement<(), _> =
e::a()
    .href("https://example.com")
    .target(attributes::Target::NewTab) // _blank
    .rel(vec![attributes::Rel::NoOpener, attributes::Rel::NoReferrer])
# ;
```

But the following wont compile:

```rust,compile_fail
# extern crate natrix;
# use natrix::prelude::*;
# let _: e::HtmlElement<(), _> =
e::div()
    .target("_blank") // error: no method named `target` found for struct `HtmlElement<_, _div>`
# ;
```

Attributes can be set by anything that implements the [`ToAttribute`](dom::ToAttribute) trait, this includes numberics, [`Option`](std::option::Option), and [`bool`](std::primitive::bool), and others.
Attributes can also be reactive as closures implement the [`ToAttribute`](dom::ToAttribute) trait.

```rust,no_run
# extern crate natrix;
# use natrix::prelude::*;
#
# #[derive(State)]
# struct MyComponent {
#     pub is_active: Signal<bool>,
# }
#
# fn render(ctx: &mut RenderCtx<MyComponent>) -> impl Element<MyComponent> {
e::button()
    .disabled(|ctx: &mut RenderCtx<MyComponent>| !*ctx.is_active.read())
    .text("Click me!")
    .on::<events::Click>(|ctx: &mut Ctx<MyComponent>, _, _| {
        *ctx.is_active.write() = !*ctx.is_active.read();
    })
# }
```

Importantly for the attribute helpers [`AttributeKind`](dom::attributes::ToAttribute::AttributeKind) determines what kind of values are allowed for that helper. Important a attribute kind of for example `bool` also supports `Option<bool>`, a closure returning `bool`, etc. For example this wont compile:
```rust,compile_fail
# extern crate natrix;
# use natrix::prelude::*;
# let _: e::HtmlElement<(), _> =
e::a()
    .target("_blank") // error: expected `attributes::Target`, found `&'static str`
# ;
```

## Classes

The [`.class`](dom::html_elements::HtmlElement::class) method is _not_ a alias for [`.attr`](dom::html_elements::HtmlElement::attr), it will add the class to the element, and not replace it. This is because the `class` attribute is a special case in HTML, and is used to apply CSS styles to elements. The [`.class`](dom::html_elements::HtmlElement::class) method will add the class to the element, and not replace any existing ones.

```rust,no_run
# extern crate natrix;
# use natrix::prelude::*;
#
const FOO: Class = natrix::class!(); // unique class name
const BAR: Class = natrix::class!();

# let _: e::HtmlElement<(), _> =
e::div()
    .class(FOO)
    .class(BAR)
# ;
```

Classes can also be reactive as closures implement the [`ToClass`](dom::ToClass) trait.

```rust
# extern crate natrix;
# use natrix::prelude::*;
#
const ACTIVE: Class = natrix::class!();

# #[derive(State)]
# struct MyComponent {
#     pub is_active: Signal<bool>,
# }
#
# fn render(ctx: &mut RenderCtx<MyComponent>) -> impl Element<MyComponent> {
e::div()
    .class(|ctx: &mut RenderCtx<MyComponent>| {
        if *ctx.is_active.read() {
            Some(ACTIVE)
        } else {
            None
        }
    })
    .child(e::button()
        .text("Click me!")
        .on::<events::Click>(|ctx: &mut Ctx<MyComponent>, _, _| {
            *ctx.is_active.write() = !*ctx.is_active.read();
        })
    )
# }
```
