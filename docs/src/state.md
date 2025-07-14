# State

State in natrix usually refers to stuff implementing the [`State`](reactivity::State) trait.
This trait is a **marker** trait, and is intended to be implemented by the `State` derive macro, which will insert bounds to assert that all your fields are also `State`.

```rust
# extern crate natrix;
# use natrix::prelude::*;

#[derive(State)]
struct App {
    counter: Signal<u8>,
}
```

For example the code below wont compile because `u8` is not `State`
```rust,compile_fail
# extern crate natrix;
# use natrix::prelude::*;

#[derive(State)]
struct App {
    counter: u8,
}
```

## Nesting state
You can easialy nest state:
```rust
# extern crate natrix;
# use natrix::prelude::*;

#[derive(State)]
struct Book {
    title: Signal<String>,
    author: Signal<String>,
}

#[derive(State)]
struct App {
    book: Book,
    user: Signal<String>,
}
```
Now you get fine-grained reactivity on the `book` fields.

> [!IMPORTANT]
> Never directly overwrite a `State`. Doing this will not trigger reactive updates. and will break your app.
> ```rust
> # extern crate natrix;
> # use natrix::prelude::*;
> #
> # #[derive(State)]
> # struct App {
> #  counter: Signal<u8>
> # }
> # fn render() -> impl Element<App> {
> e::button().on::<events::Click>(|mut ctx: EventCtx<App>, _|{
>   // This is really bad:
>   ctx.counter = Signal::new(10);
> })
> # }
> ```
> This also includes overwriting any `State` struct directly, like `ctx.book = Book::...`

## `Signal`
the [`Signal`](prelude::Signal) is the core reactive primitive in natrix, and implements read and write tracking on derefrencing.

```rust
# extern crate natrix;
# use natrix::prelude::*;

#[derive(State)]
struct App {
    counter: Signal<u8>,
}

fn render() -> impl Element<App> {
    e::button()
        .text(|ctx: RenderCtx<App>| *ctx.counter) // The `*` read the `u8` value and tells natrix to track this
        .on::<events::Click>(|mut ctx: EventCtx<App>, _| {
            // Similarly this informs natrix the signal changed.
            *ctx.counter += 1;
        })
}

```


