# features

## opt-in features

### `nightly`

this feature enables nightly only features. this includes:

#### Default types for [`EmitMessage`](reactivity::component::Component::EmitMessage) and [`ReceiveMessage`](reactivity::component::Component::ReceiveMessage)

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

### `keep_console_in_release`

By default natrix strips out all console logs, including your own, in release builds. (including from panics)
This feature disables that, allowing you to see the console logs in release builds.

## Internal features

You might notice a few `_internal_*` features listed for `natrix` itself, and you'll also see `_natrix_internal_*` proxy features in your own crate's `Cargo.toml`. These are internal features, and as such, we won't be documenting their specific functionalities in detail.

These features are primarily used by `natrix-cli` to build special versions of your application for bundling reasons, such as CSS extraction or Static Site Generation (SSG). The `_natrix_internal_*` entries in your `Cargo.toml` act as "feature proxies," allowing the bundler to correctly apply these configurations during the build process without needing to modify your project's manifest directly.

If you are migrating an existing project to a newer Natrix version, it's recommended to generate a new Natrix project. You can then copy over any new `_natrix_internal_*` feature proxies from the generated `Cargo.toml` into your existing project to ensure compatibility with the latest bundler requirements.
