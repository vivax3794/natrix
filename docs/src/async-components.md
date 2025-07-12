# Async

Async is a really important part of any web application, as it's how you do IO and talk to other services or your backend.
Natrix provides [`AsyncCtxHandle`](reactivity::state::AsyncCtxHandle), via the [`.use_async`](prelude::EventCtx::use_async) Method.

## What is a `AsyncCtxHandle`?

Internally natrix stores the state as a `Rc<RefCell<...>>`, `AsyncCtxHandle` is a wrapper around a [`Weak<...>`](std::rc::Weak) version of the same state, that exposes a limited safe api to allow you to borrow the state at arbitrary points in the code.

The main method on a async handle is the [`.update`](reactivity::state::AsyncCtxHandle::update) method, which allows you to borrow the state mutably. This returns a `Option<...>`, if this returns [`None`](std::option::Option::None), then the component is dropped and you should in most case return/cancel the current task.

On borrowing (via [`.update`](reactivity::state::AsyncCtxHandle::update)) the framework will clear the reactive state of signals, and will trigger a reactive update on closure return. (i.e the framework will keep the UI in sync with changes). But this also means you should not borrow this in a loop, and should prefer to borrow it for the maximum amount of time that doesn't hold it across a yield point.

## `.use_async`

The [`.use_async`](prelude::EventCtx::use_async) method takes a async closure and schedules it to run with a [`AsyncCtxhandle`](reactivity::state::AsyncCtxHandle) borrowed from the state. The closure should return `Option<()>`, This is to allow use of `?` to return early if the component is dropped.

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
        .on::<events::Click>(|mut ctx: EventCtx<HelloWorld>, _| {
            ctx.use_async(async |ctx| {
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

## Lenses
Lenses used in `.use_async` are required to be [`AsyncSafe`](lens::AsyncSafe), most lenses are, except those created by `ctx.guard` and [`.map`](lens::Lens::map).
This is because the lenses from `ctx.guard` are only valid as long as the framework is able to prevent their usage (which it does via the hook ordering guarantees), but said guarantees dont apply for async code.

The following example will (correctly) fail to compile:
```rust,compile_fail
# extern crate natrix;
# use natrix::prelude::*;
fn use_string<S: State>(subs: impl Lens<S, u8>) -> impl Element<S> {
    e::button().on::<events::Click>(move |mut ctx: EventCtx<S>, _| {
        let subs = subs.clone();
        ctx.use_async(async move |ctx| {
            ctx.update(move |mut ctx| {
                *ctx.get(subs) = 100;
            })?;
            Some(())
        });
    })
}
```
To make the above compile add a `AsyncSafe` bound, which will enforce callers dont use `ctx.guard` and similar to create the lens:
```rust
# extern crate natrix;
# use natrix::prelude::*;
fn use_string<S: State>(subs: impl Lens<S, u8> + lens::AsyncSafe) -> impl Element<S> {
    e::button().on::<events::Click>(move |mut ctx: EventCtx<S>, _| {
        let subs = subs.clone();
        ctx.use_async(async move |ctx| {
            ctx.update(move |mut ctx| {
                *ctx.get(subs) = 100;
            })?;
            Some(())
        });
    })
}
```

Now trying to use a `ctx.guard` lens with this helper will fail:
```rust,compile_file
# extern crate natrix;
# use natrix::prelude::*;
# fn use_string<S: State>(subs: impl Lens<S, u8> + lens::AsyncSafe) -> impl Element<S> {
#    e::button().on::<events::Click>(move |mut ctx: EventCtx<S>, _| {
#        let subs = subs.clone();
#        ctx.use_async(async move |ctx| {
#            ctx.update(move |mut ctx| {
#                *ctx.get(subs) = 100;
#            })?;
#            Some(())
#        });
#    })
# }
fn uses_guard<S: State>(option: impl Lens<S, Option<u8>> + lens::AsyncSafe) -> impl Element<S> {
    |ctx: &mut RenderCtx<S>| {
        if let Some(guard) = ctx.guard(option) {
            // Even tho the argument is async-safe this isnt anymore 
            use_string(guard).render()
        } else {
            "It is none!".render()
        }
    }
}
```

> [!TIP]
> If you need to use `.map` for a `AsyncSafe` lens there is [`.map_assert_async_safe`](lens::Lens::map_assert_async_safe).
> Crucially this does not actually perform any checks that the closure is in fact async safe.
> Its up to you to not write panicky code.
