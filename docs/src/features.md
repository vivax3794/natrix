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
#        e::button().on::<events::Click>(|ctx: E<Self>, _|{
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

## default features
For most complex applications you will likely need all the default features.
But they can be disabled if you want to reduce compile times or binary size.

### `scoped_css`
This enables [Scoped Css](css.md#scoped-css). 
This pulls in `lightningcss` *in the proc-macro*.
As such disabling this feature will result in faster compile times.

### `inline_css`
This enables [Inline Css](css.md#inline-css). This doesnt have the same compile time impact as `scoped_css` as it only pulls in `data-encoding` in the proc-macro, but it will still impact compile times.

### `panic_hook`

This feature enables a panic hook that is auto installed when using [`mount`](component::mount) (or can be set manually with [`natrix::set_panic_hook`](set_panic_hook)), this panic hook will prevent any further rust code from running if a panic happens, which prevents undefined behaviour.

On the default `natrix new` project (on nightly), a normal build is 30KB while a build without this feature is 22KB.

> [!DANGER]
> Disabling this should be considered `unsafe`, and is an assertion from you that your code will never panic.
>
> This will actually make `natrix build` strip out all branches that panic, which means hitting those branches is **undefined behaviour**.

## auto nightly

natrix will auto detect when its compiled on nightly and use certain (non-public-facing) features. this is one of the reasons its recommended to use nightly rust.

- optimize text updates, on stable updating a text node is done via `replace_child`, on nightly it uses `set_text_content`
