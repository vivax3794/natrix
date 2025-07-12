# Async

Async is a really important part of any web application, as it's how you do IO and talk to other services or your backend.
Natrix provides [`DeferredCtx`](reactivity::state::DeferredCtx), via the [`.deferred_borrow`](prelude::EventCtx::deferred_borrow) method, to facilitate this. as well as the [`.use_async`](prelude::EventCtx::use_async) helper.

## What is a `DeferredCtx`?

Internally natrix stores the state as a `Rc<RefCell<...>>`, [`DeferredCtx`](reactivity::state::DeferredCtx) is a wrapper around a [`Weak<...>`](std::rc::Weak) version of the same state, that exposes a limited safe api to allow you to borrow the state at arbitrary points in the code, usually in async functions.

The main method on a deferred context is the [`.update`](reactivity::state::DeferredCtx::update) method, which allows you to borrow the state mutably. This returns a `Option<...>`, if this returns [`None`](std::option::Option::None), then the component is dropped and you should in most case return/cancel the current task.

On borrowing (via [`.update`](reactivity::state::DeferredCtx::update)) the framework will clear the reactive state of signals, and will trigger a reactive update on closure return. (i.e the framework will keep the UI in sync with changes). But this also means you should not borrow this in a loop, and should prefer to borrow it for the maximum amount of time that doesn't hold it across a yield point.

### Example

```rust
# extern crate natrix;
# use natrix::prelude::*;
# use natrix::reactivity::state::DeferredCtx;
#
# async fn foo() {}
#
#[derive(State)]
struct HelloWorld {
    counter: Signal<u8>,
}

async fn use_context(mut ctx: DeferredCtx<HelloWorld>) {
    if ctx.update(|mut ctx| {
        *ctx.counter += 1;
        *ctx.counter += 1;
    }).is_none() {return;}; 

    foo().await;

    if ctx.update(|mut ctx| {
        *ctx.counter += 1;
    }).is_none() {return;}; 
}
```

## `.use_async`

In most cases where you have use for a [`DeferredCtx`](reactivity::state::DeferredCtx) it will be in a async function.
The [`.use_async`](prelude::EventCtx::use_async) method is a wrapper that takes a async closure and schedules it to run with a [`DeferredCtx`](reactivity::state::DeferredCtx) borrowed from the state. The closure should return `Option<()>`, This is to allow use of `?` to return early if the component is dropped.

```rust
# extern crate natrix;
# use natrix::prelude::*;
#
# async fn foo() {}
#
#[derive(State)]
struct HelloWorld {
    counter: Signal<u8>,
}

fn render_hello_world() -> impl Element<HelloWorld> {
    e::button()
        .text(|ctx: &mut RenderCtx<HelloWorld>| *ctx.counter)
        .on::<events::Click>(|mut ctx: EventCtx<HelloWorld>, token, _| {
            ctx.use_async(token, async |ctx| {
                ctx.update(|mut ctx| {
                    *ctx.counter += 1;
                })?;

                foo().await;

                ctx.update(|mut ctx| {
                    *ctx.counter += 1;
                })?;

                Some(())
            });
        })
}
```
