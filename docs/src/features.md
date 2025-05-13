# features


## opt-in features

### `nightly`

this feature enables nightly only features. this includes:

#### Default types for [`EmitMessage`](component::Component::EmitMessage) and [`ReceiveMessage`](component::Component::ReceiveMessage)

this allows you to omit these in your trait implementation, which is really nice.

```rust
# extern crate natrix;
# use natrix::prelude::*;
#
# #[derive(Component)]
# struct Example;
#
impl Component for Example{
    fn render() -> impl Element<Self> {
        e::div()
            .text("hello world")
    }
}
```

#### `must_not_suspend`

this annotates certain framework structs as [`must_not_suspend`](https://github.com/rust-lang/rust/issues/83310), which lets rust warn you if you misuse them in async contexts.

> [!IMPORTANT]
> this requires your project to also enable the feature and use the lint
>
> ```rust
> #![feature(must_not_suspend)]
> #![warn(must_not_suspend)]
> ```

### `async_utils`

adds the [`async_utils`] module which contains stuff like a wasm compatible [`sleep`](async_utils::sleep) function.

```rust
# extern crate natrix;
use std::time::Duration;
async fn foo() {
    natrix::async_utils::sleep(Duration::from_secs(1)).await;
}
```

### `ergonomic_ops`

Implements `AddAssign`, `SubAssign`, etc on signals, allowing you to omit the dereference in certain situations.
This is disabled by default because it does not match other smart pointers, and still requires the dereference in certain situations.

```rust
# extern crate natrix;
# use natrix::prelude::*;
# #[derive(Component)]
# struct Hello { counter: u8 }
# impl Component for Hello {
#     fn render() -> impl Element<Self> {
#        e::button().on::<events::Click>(|ctx: E<Self>, _, _|{
// Without `ergonomic_ops`
*ctx.counter += 1;
*ctx.counter = *ctx.counter + 1;

// With `ergonomic_ops`
ctx.counter += 1;
*ctx.counter = *ctx.counter + 1;
# })
# }}
```

Notice how we still need the dereference for a plain assignment and addition? This inconsistency is why this feature is disabled by default as many might find this confusing.

### `either`

Implements [`Component`](component::Component) and [`ToAttribute`](html_elements::ToAttribute) for [`Either`](https://docs.rs/either/latest/either/enum.either.html) from the `either` crate.

### `keep_console_in_release`
By default natrix strips out all console logs, including your own, in release builds. (including from panics)
This feature disables that, allowing you to see the console logs in release builds.

## default features
For most complex applications you will likely need all the default features.
But they can be disabled if you want to reduce compile times or binary size.
