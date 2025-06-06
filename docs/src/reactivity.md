# Reactivity

## Callbacks

You have already seen `|ctx: R<Self>| ...` used in the varying examples in the book.
Lets go into some more detail about what this does.

> [!TIP]
> If you're looking to create components without state that still leverage natrix's reactivity system, see the [Stateless Components](reactivity::stateless-components.md) documentation.

The callbacks return a value that implements [`Element`](dom::element::Element), internally the framework will register which fields you accessed.
And when those fields change, the framework will recall the callback and update the element with the result.

Natrix does **_not_** use a virtual dom, meaning when a callback is re-called the framework will swap out the entire associated element.
See below for tools to mitigate this.

> [!IMPORTANT]
> Natrix assumes render callbacks are pure, meaning they do not have side effects.
> If you use stuff like interior mutability it will break framework assumptions,
> and will likely lead to panics or desynced state.
> For example using interior mutability to hold onto a [`Guard`](reactivity::state::Guard) outside its intended scope will invalidate its guarantees.

### Execution Guarantees

Natrix _only_ makes the following guarantees about when a callback will be called:

- It will not be called if a parent is dirty.

Thats, natrix does not make any guarantees about the order of sibling callbacks.
Natrix guarantees around how often a value is called is... complex because of features such as `.watch`, in general the reactive features below should be mainly treated as very strong hints to the framework, and optimizations might cause various use cases to result in more or less calls.

## `.watch`

Now imagine you only access part of a field.

```rust
# extern crate natrix;
# use natrix::prelude::*;
# #[derive(Component)]
# struct HelloWorld {
#     counter: u8,
# }
# impl Component for HelloWorld {
#     fn render() -> impl Element<Self> {
e::div()
    .child(|ctx: R<Self>| {
        format!("{}", *ctx.counter > 10)
    })
#      }
# }
```

This will work, but it will cause the callback to be called every time `counter` changes, even if it causes no change to the dom.
In this case thats fine, its not a expensive update, but imagine if this was a expensive operation.
This is where [`.watch`](reactivity::state::RenderCtx::watch) comes in.

What it does is cache the result of a callback, and then it calls it on any change, but will compare the new value to the old value.
And only re-runs the surrounding callback if the value has changed.

```rust
# extern crate natrix;
# use natrix::prelude::*;
# #[derive(Component)]
# struct HelloWorld {
#     counter: u8,
# }
# impl Component for HelloWorld {
#     fn render() -> impl Element<Self> {
e::div()
    .child(|ctx: R<Self>| {
        let bigger_than_10 = ctx.watch(|ctx| *ctx.counter > 10);
        format!("{}", bigger_than_10)
    })
#      }
# }
```

| Action         | `ctx.watch(...)` runs | `format!(...)` runs |
| -------------- | --------------------- | ------------------- |
| initial render | yes                   | yes                 |
| `0` -> `1`     | yes                   | no                  |
| `10` -> `11`   | yes                   | yes                 |
| `11` -> `12`   | yes                   | no                  |
| `11` -> `10`   | yes                   | yes                 |

This can be more usefully used for example when dealing with a [`Vec`](std::vec::Vec) of items.
For example `ctx.watch(|ctx| ctx.items[2])`

## `guard_...`

### Problem

Now you could imagine `.watch` being useful for [`Option`](std::option::Option) and [`Result`](std::result::Result) types.
It is, but you have to be careful about how you use it.

```rust
# extern crate natrix;
# use natrix::prelude::*;
# #[derive(Component)]
# struct HelloWorld {
#     option: Option<u8>,
# }
# impl Component for HelloWorld {
#     fn render() -> impl Element<Self> {
e::div()
    .child(|ctx: R<Self>| {
        if ctx.watch(|ctx| ctx.option.is_some()) {
            let value = ctx.option.unwrap();
            e::h1()
                .text(format!("Value: {}", value))
                .into_generic()
        } else {
            "None".into_generic()
        }
    })
#      }
# }
```

This will work, but it will still cause the callback to be called every time `option` changes because it still uses `ctx.option` directly.
As with any fine-grained reactivity, you can use nested callbacks to get a better effect.

```rust
# extern crate natrix;
# use natrix::prelude::*;
# #[derive(Component)]
# struct HelloWorld {
#     option: Option<u8>,
# }
# impl Component for HelloWorld {
#     fn render() -> impl Element<Self> {
e::div()
    .child(|ctx: R<Self>| {
        if ctx.watch(|ctx| ctx.option.is_some()) {
            e::h1()
                .text(|ctx: R<Self>| ctx.option.unwrap())
                .into_generic()
        } else {
            "None".into_generic()
        }
    })
#      }
# }
```

And this does work exactly as we want it to. But there is one downside.
That `.unwrap`, we know for a fact that it will never panic because of the condition above.
But because in rusts eyes these callbacks are not isolated we cant prove to it otherwise.
In addition that unwrap means you cant use the recommended lint to deny them.

### Solution

This is where the `guard_option` (and `guard_result`) macros come in.
They use `ctx.watch` internally, and gives you a way to access the value without having to unwrap it.

```rust
# extern crate natrix;
# use natrix::prelude::*;
use natrix::guard_option;

# #[derive(Component)]
# struct HelloWorld {
#     option: Option<u8>,
# }
# impl Component for HelloWorld {
#     fn render() -> impl Element<Self> {
e::div()
    .child(|ctx: R<Self>| {
        if let Some(guard) = guard_option!(|ctx| ctx.option.as_ref()) {
            e::h1()
                .text(move |ctx: R<Self>| *ctx.get(&guard))
                .into_generic()
        } else {
            "None".into_generic()
        }
    })
#      }
# }
```

This will work exactly the same as the previous example, but hides the `.unwrap()` from the user.

> [!WARNING]
> Similarly to [`DeferredRef`](reactivity::state::DeferredRef) you should not hold this across a yield point.
> `guard_option` does in fact still use `.unwrap()` internally, meaning its effectively the same as the "bad" code above.
> It is simply a nice api that enforces the invariant that you only `.unwrap` in a context where you have done the `.is_some()` check in a parent hook.

## [`List`](dom::list::List)

You often have to render a list of items, and doing that in a reactive way is a bit tricky.
The [`List`](dom::list::List) element is a way to do this.

```rust
# extern crate natrix;
use natrix::prelude::*;
use natrix::reactivity::State;
use natrix::dom::List;

#[derive(Component)]
struct HelloWorld {
    items: Vec<u8>,
}

impl Component for HelloWorld {
    fn render() -> impl Element<Self> {
        e::div()
            .child(List::new(
            |ctx: &State<Self>| &ctx.items,
            |_ctx: R<Self>, getter| {
                e::div().text(move |ctx: R<Self>| getter.get_watched(ctx))
            }
        ))
    }
}
```

See the docs in the [`List`](dom::list::List) module for more details.

