# Async

Async is a really important part of any web application, as its how you do IO and talk to other services or your backend.
Natrix provides [`DeferredCtx`](state::DeferredCtx), via the [`.deferred_borrow`](state::State::deferred_borrow) method, to facilitate this. as well as the [`.use_async`](state::State::use_async) helper.

## What is a `DeferredCtx`?

Internally natrix stores the state as a `Rc<RefCell<...>>`, [`DeferredCtx`](state::DeferredCtx) is a wrapper around a [`Weak<...>`](std::rc::Weak) version of the same state, that exposes a limited safe api to allow you to borrow the state at arbitrary points in the code, usually in async functions.

The main method on a deferred context is the [`.borrow_mut`](state::DeferredCtx::borrow_mut) method, which allows you to borrow the state mutably. This returns a [`Option<DeferredRef>`](state::DeferredRef) which internally holds both a strong [`Rc`](std::rc::Rc) and a [`RefMut`](std::cell::RefMut) into the state.
If this returns [`None`](std::option::Option::None), then the component is dropped and you should in most case return/cancel the current task.

> [!IMPORTANT]
> Holding a [`DeferredRef`](state::DeferredRef) across a yield point (holding across `.await`) is considered a bug, and will likely lead to a panic on debug builds, and desynced state on release builds.

On borrowing (via [`.borrow_mut`](state::DeferredCtx::borrow_mut)) the framework will clear the reactive state of signals, and will trigger a reactive update on drop (i.e the framework will keep the UI in sync with changes made via this borrow). But this also means you should not borrow this in a loop, and should prefer to borrow it for the maximum amount of time that doesnt hold it across a yield point.

### Bad Example

```rust
# extern crate natrix;
# use natrix::prelude::*;
# use natrix::state::DeferredCtx;
#
# async fn foo() {}
#
#[derive(Component)]
 struct HelloWorld {
     counter: u8,
}

# impl Component for HelloWorld {
#     fn render() -> impl Element<Self> {
#         e::div()
#     }
# }
#
async fn use_context(mut ctx: DeferredCtx<HelloWorld>) {
    let mut borrow = ctx.borrow_mut().unwrap(); // Bad, we are panicking instead of returning.
    *borrow.counter += 1;

    drop(borrow); // Bad we are triggering multiple updates.
    let mut borrow = ctx.borrow_mut().unwrap();

    *borrow.counter += 1;
    foo().await; // Bad we are holding the borrow across a yield point.
    *borrow.counter += 1;
}
```

### Good Example

```rust
# extern crate natrix;
# use natrix::prelude::*;
# use natrix::state::DeferredCtx;
#
# async fn foo() {}
#
#[derive(Component)]
 struct HelloWorld {
     counter: u8,
}

# impl Component for HelloWorld {
#     fn render() -> impl Element<Self> {
#         e::div()
#     }
# }
#
async fn use_context(mut ctx: DeferredCtx<HelloWorld>) {
    { // Scope the borrow
        let Some(mut borrow) = ctx.borrow_mut() else {
            return;
        };
        *borrow.counter += 1;
        *borrow.counter += 1;
    } // Borrow is dropped here, triggering a reactive update.

    foo().await;

    let Some(mut borrow) = ctx.borrow_mut() else {
        return;
    };
    *borrow.counter += 1;
}
```

In other words, you should consider [`.borrow_mut`](state::DeferredCtx::borrow_mut) to be a similar to [`Mutex::lock`](std::sync::Mutex::lock) in terms of scoping and usage. You should not hold the borrow across a yield point, and you should not hold it for longer than necessary.

## `.use_async`

In most cases where you have use for a [`DeferredCtx`](state::DeferredCtx) it will be in a async function.
The [`.use_async`](state::State::use_async) method is a wrapper that takes a async closure and schedules it to run with a [`DeferredCtx`](state::DeferredCtx) borrowed from the state. The closure should return `Option<()>`, This is to allow use of `?` to return early if the component is dropped.

```rust
# extern crate natrix;
# use natrix::prelude::*;
#
# async fn foo() {}
#
#[derive(Component)]
struct HelloWorld {
    counter: u8,
}

impl Component for HelloWorld {
    fn render() -> impl Element<Self> {
        e::button()
            .text(|ctx: R<Self>| *ctx.counter)
            .on::<events::Click>(|ctx: E<Self>, token, _| {
                ctx.use_async(token, async |ctx| {
                    {
                        let mut borrow = ctx.borrow_mut()?;
                        *borrow.counter += 1;
                    }

                    foo().await;
                    let mut borrow = ctx.borrow_mut()?;
                    *borrow.counter += 1;

                    Some(())
                });
            })
    }
}
```
