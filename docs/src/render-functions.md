# Render Functions

Render functions are the core of how you build UI in Natrix. They are functions that return `impl Element<T>` where `T` is your state type. This chapter explains how to write and compose render functions effectively.

## Basic Render Functions

Every Natrix application starts with a render function. Elements are generic over the state type it will work with:

```rust,no_run
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
    natrix::mount(MyApp { count: Signal::new(0) }, render_my_app);
}
```

## Passing Arguments

Since render functions are just regular Rust functions, they can take arguments:

```rust
# extern crate natrix;
# use natrix::prelude::*;


#[derive(State)]
struct MyApp {
    user_name: Signal<String>,
}

fn greeting(name: &'static str) -> impl Element<MyApp> {
    e::h1().text("Hello ").text(name)
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


#[derive(State)]
struct MyApp {
    user_name: Signal<String>,
}

fn greeting(name: impl Element<MyApp>) -> impl Element<MyApp> {
    e::h1().text("Hello ").child(name)
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


#[derive(State)]
struct App {
    counter: Signal<i32>,
}

fn fancy_button(
    text: &'static str,
    on_click: impl EventHandler<App, events::Click>,
) -> impl Element<App> {
    e::button()
        .text(text)
        .on(on_click)
}

fn render_counter() -> impl Element<App> {
    e::div()
        .child(e::p().text(|ctx: &mut RenderCtx<App>| *ctx.counter))
        .child(fancy_button("Increment", |ctx: &mut Ctx<App>, _, _| {
            *ctx.counter += 1;
        }))
}
```

## Direct state access
Since render functions are specialized on the state type, you can access the fields directly.

```rust
# extern crate natrix;
# use natrix::prelude::*;

#[derive(State)]
struct App {
    username: Signal<String>,
}

fn greeting() -> impl Element<App> {
    e::h1()
        .text("Hello ")
        .text(|ctx: &mut RenderCtx<App>| ctx.username.clone())
}

fn render_app() -> impl Element<App> {
    e::div()
        .child(greeting())
        .child(e::p().text("Cool right?"))
}
```

## Generic state access.
What if you want a more reusable component, lets say a slider?
For that we have lenses, the primary api for lenses is the `lens!` macro and [`Lens`](lens::Lens) trait methods.
The signature for a lens is `Lens<Source, Target>`.

```rust
# extern crate natrix;
# use natrix::prelude::*;

#[derive(State)]
struct App {
    first: Signal<u8>,
    second: Signal<u8>,
}

fn counter(value: impl Lens<App, u8> + Copy) -> impl Element<App> {
    e::div()
        .child(e::button().text("-").on::<events::Click>(move |ctx: &mut Ctx<App>, _, _| {
            *ctx.get(value) -= 1;
        }))
        .text(move |ctx: &mut RenderCtx<App>| *ctx.get(value))
        .child(e::button().text("+").on::<events::Click>(move |ctx: &mut Ctx<App>, _, _| {
            *ctx.get(value) += 1;
        }))
}

fn render_app() -> impl Element<App> {
    e::div()
        .child(counter(lens!(App => .first).deref()))
        .child(counter(lens!(App => .second).deref()))
}
```
The amazing thing is that since the helper is still concrete (`impl Element<App>`), you can still access any field directly, if you said had a `theme` field, the `counter` function could access that directly without needing lenses.

> [!TIP]
> All lenses are `Clone`, but you can require them to be `Copy` for ergonomics if you know the lenses you use are `Copy`.
> (Only lenses that capture non-`Copy` data in closure via say [`.map`](lens::Lens::map) are not `Copy`)

## Generic State Store
If you are writing a component library you naturally wont know the state, then you can simply use a generic:
```rust
# extern crate natrix;
# use natrix::prelude::*;

fn counter<S: State>(value: impl Lens<S, u8> + Copy) -> impl Element<S> {
    e::div()
        .child(e::button().text("-").on::<events::Click>(move |ctx: &mut Ctx<S>, _, _| {
            *ctx.get(value) -= 1;
        }))
        .text(move |ctx: &mut RenderCtx<S>| *ctx.get(value))
        .child(e::button().text("+").on::<events::Click>(move |ctx: &mut Ctx<S>, _, _| {
            *ctx.get(value) += 1;
        }))
}
```

## Trait bound on state.
What if you want the nice feature of always having access to global state, but in a generic component library? well you can use traits!
```rust
# extern crate natrix;
use natrix::prelude::*;

mod my_library {
    use natrix::prelude::*;
    pub trait HasLanguage {
        fn get_language(&self) -> &'static str;
    }

    pub fn greeting<S>(name: impl Lens<S, String>) -> impl Element<S>
        where S: State + HasLanguage
    {
        e::div()
            .text(|ctx: &mut RenderCtx<S>| {
                match ctx.get_language() {
                    "NO" => "Hallo ",
                    "EN" | _ => "Greetins "
                }
            })
            .text(move |ctx: &mut RenderCtx<S>| ctx.get(name.clone()).clone())
    }
}

#[derive(State)]
struct App {
    lang: Signal<&'static str>,
    author_name: Signal<String>,
}

impl my_library::HasLanguage for App {
    fn get_language(&self) -> &'static str {
        &self.lang
    }
}

fn render_app() -> impl Element<App> {
    e::div()
        .child(my_library::greeting(lens!(App => .author_name).deref()))
}
```
