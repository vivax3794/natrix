# Render Functions

Render functions are the core of how you build UI in Natrix. They are functions that return `impl Element<T>` where `T` is your state type. This chapter explains how to write and compose render functions effectively.

## Basic Render Functions

Every Natrix application starts with a render function. The function is generic over the state type it will work with:

```rust
# extern crate natrix;
# use natrix::prelude::*;

#[derive(State)]
struct MyApp {
    count: Signal<i32>,
}

fn render_my_app() -> impl Element<MyApp> {
    e::div()
        .child(e::h1().text("My App"))
        .child(e::p().text(|ctx: &mut RenderCtx<MyApp>| *ctx.count))
}

fn main() {
    natrix::mount(MyApp { count: Signal::new(0) }, render_my_app());
}
```

## Reusable Render Functions

You can create reusable render functions by making them generic over the state type. This allows you to use the same UI component with different state types:

```rust
# extern crate natrix;
# use natrix::prelude::*;

fn hello<T: State>() -> impl Element<T> {
    e::h1().text("Hello World")
}

#[derive(State)]
struct MyApp;

fn render_my_app() -> impl Element<MyApp> {
    e::div()
        .child(hello())
        .child(e::p().text("Welcome to Natrix"))
}
```

## Passing Arguments

Since render functions are just regular Rust functions, they can take arguments:

```rust
# extern crate natrix;
# use natrix::prelude::*;

fn greeting<T: State>(name: &str) -> impl Element<T> {
    e::h1().text("Hello ").text(name)
}

#[derive(State)]
struct MyApp {
    user_name: Signal<String>,
}

fn render_my_app() -> impl Element<MyApp> {
    e::div()
        .child(greeting("World"))
        .child(e::p().text(|ctx: &mut RenderCtx<MyApp>| ctx.user_name.clone()))
}
```

## Reactive Arguments

For reactive content, pass `impl Element<T>` as arguments instead of raw values. This allows fine-grained reactivity:

```rust
# extern crate natrix;
# use natrix::prelude::*;

fn greeting<T: State>(name: impl Element<T>) -> impl Element<T> {
    e::h1().text("Hello ").child(name)
}

#[derive(State)]
struct MyApp {
    user_name: Signal<String>,
}

fn render_my_app() -> impl Element<MyApp> {
    e::div()
        .child(greeting(|ctx: &mut RenderCtx<MyApp>| ctx.user_name.clone()))
}
```

Now only the name part will update when `user_name` changes, not the entire greeting.

## Event Handlers

You can pass event handlers to render functions using the [`EventHandler`](dom::events::EventHandler) trait:

```rust
# extern crate natrix;
# use natrix::prelude::*;
use natrix::dom::EventHandler;

fn fancy_button<T: State>(
    text: &str,
    on_click: impl EventHandler<T, events::Click>,
) -> impl Element<T> {
    e::button()
        .text(text)
        .on(on_click)
}

#[derive(State)]
struct Counter {
    value: Signal<i32>,
}

fn render_counter() -> impl Element<Counter> {
    e::div()
        .child(e::p().text(|ctx: &mut RenderCtx<Counter>| *ctx.value))
        .child(fancy_button("Increment", |ctx: &mut Ctx<Counter>, _, _| {
            *ctx.value += 1;
        }))
}
```

## State-Specific Render Functions

You can also write render functions that are specific to a particular state type. This is useful for complex components:

```rust
# extern crate natrix;
# use natrix::prelude::*;

#[derive(State)]
struct Counter {
    value: Signal<i32>,
}

fn increment_button(delta: i32) -> impl Element<Counter> {
    e::button()
        .text(format!("{:+}", delta))
        .on::<events::Click>(move |ctx: &mut Ctx<Counter>, _, _| {
            *ctx.value += delta;
        })
}

fn render_counter() -> impl Element<Counter> {
    e::div()
        .child(increment_button(-10))
        .child(increment_button(-1))
        .child(e::span().text(|ctx: &mut RenderCtx<Counter>| *ctx.value))
        .child(increment_button(1))
        .child(increment_button(10))
}
```

## Composing Render Functions

You can compose render functions together to build complex UIs:

```rust
# extern crate natrix;
# use natrix::prelude::*;

fn header<T: State>() -> impl Element<T> {
    e::header()
        .child(e::h1().text("My App"))
        .child(e::nav().text("Navigation"))
}

fn footer<T: State>() -> impl Element<T> {
    e::footer()
        .child(e::p().text("© 2024 My App"))
}

#[derive(State)]
struct MyApp {
    content: Signal<String>,
}

fn render_my_app() -> impl Element<MyApp> {
    e::div()
        .child(header())
        .child(e::main().text(|ctx: &mut RenderCtx<MyApp>| ctx.content.clone()))
        .child(footer())
}
```

## Best Practices

1. **Keep render functions pure**: They should only depend on their arguments and not have side effects.
2. **Use generic functions for reusability**: Make functions generic over `T: State` when they don't need specific state.
3. **Pass reactive content as `impl Element<T>`**: This enables fine-grained reactivity.
4. **Use descriptive names**: Function names should clearly indicate what UI they create.
5. **Break down complex UIs**: Split large render functions into smaller, composable pieces.