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

### `default_app`
This feature flag is enabled by default in the project template. And is a collection of features considered "default" for applications.
We opted for this over normal default cargo features because we think it is important for libraries to use the minimal amount of features.
Libraries should *never* enable this feature flag.
The intent is that even if a library uses all the below features it should not be tempted to simply use `default-features = true`.

* `console_log`
* `async`
* `performance_lints`

### `console_log`
Automatically sets up [`console_log`](https://crates.io/crates/console_log) on [`mount`](reactivity::component::mount).

### `async`
Enables the use of [`ctx.use_async`](reactivity::state::State::use_async) 

### `async_utils`
Enables the various async wrappers for browser apis in [`async_utils`](async_utils)

### `test_utils`
Various testing utilities, this should be enabled via a `[dev-dependencies]`.
```toml
[dev-dependencies]
natrix = {version = "*", features=["test_utils"]}
```

### `performance_lints`
This enables various warning for situations that are not bugs, or even indicate wrongful use. But are signs of poor optimization.
This features flag is extremely oppinonated about whats sane limits for various internal values.

> [!NOTE]
> Performance lints are only checked in debug builds.

## Internal features

You might notice a few `_internal_*` features listed for `natrix` itself, and you'll also see `_natrix_internal_*` proxy features in your own crate's `Cargo.toml`. These are internal features, and as such, we won't be documenting their specific functionalities in detail.

These features are primarily used by `natrix-cli` to build special versions of your application for bundling reasons, such as CSS extraction or Static Site Generation (SSG). The `_natrix_internal_*` entries in your `Cargo.toml` act as "feature proxies," allowing the bundler to correctly apply these configurations during the build process without needing to modify your project's manifest directly.

If you are migrating an existing project to a newer Natrix version, it's recommended to generate a new Natrix project. You can then copy over any new `_natrix_internal_*` feature proxies from the generated `Cargo.toml` into your existing project to ensure compatibility with the latest bundler requirements.
