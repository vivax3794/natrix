# features

## opt-in features

### `nightly`
this feature enables nightly only features. this includes:

#### default types for `emitmessage` and `receivemessage`
this allows you to ommit these in your trait implementation, which is really nice.
```rust
# extern crate natrix;
# use natrix::prelude::*;

#[derive(component)]
struct hello;

impl component {
    // no need to specify the types here
    fn render() -> impl element<self> {
        // ...
        # e::div()
        #    .text("hello world")
    }
}
```

#### `must_not_suspend`
this annotates certain framework structs as [`must_not_suspend`](https://github.com/rust-lang/rust/issues/83310), which lets rust warn you if you missuse them in async contexts. this requires your project to also enable the feature and use the lint
```rust
#![feature(must_not_suspend)]
#![warn(must_not_suspend)]
```

### `async_utils`
adds the [`async_utils`](https://docs.rs/natrix/latest/natrix/async_utils/) module which contains stuff like a wasm compatiable `sleep` function.
```rust
# extern crate natrix;
use std::time::duration;
async foo() {
    natrix::async_utils::sleep(duration::from_secs(1)).await;
}
```

### `ergonomic_ops`
Implements `Add`, `AddAssign`, `Sub`, etc on signals, allowing you to omit the dereference in certain situations.
This is disabled by default because it does not match other smart pointers, and still requires the derference in certain situations.
```rust
# extern crate natrix;
# use natrix::prelude::*;
# #[derive(component)]
# struct hello { counter: u8 }
# impl component for hello {
#     fn render() -> impl element<self> {
#        e::button().on<events::Click>(|ctx: &mut S<Self>, _|{
// Without `ergonomic_ops`
*ctx.counter += 1;
*ctx.counter = *ctx.counter + 1;

// With `ergonomic_ops`
ctx.counter += 1;
*ctx.counter = ctx.counter + 1;
# })
# }}
```
Notice how we still need the derefence for a plain assignment? This inconsistency is why this feature is disabled by default as many might find this confusing.

### `either`
implements `component` and `toattribute` for [`either`](https://docs.rs/either/latest/either/enum.either.html) from the `either` crate.

## default features

### `panic_hook`
this feature enables a panic hook that is auto installed when using `mount` (or can be set manually with `natrix::set_panic_hook`), this panic hook will prevent any futher rust code from running if a panic happens, which prevents undefined behaviour.

> [!EXAMPLE]
> On the default `natrix new` project (on nightly), a normal build is 30KB while a build without this feature is 22KB.

> [!DANGER]
> Disabling this should be considerd `unsafe`, and is an assertion from you that your code will never panic.
> 
> This will actually make `natrix build` strip out all branches that panic, which means hitting those branches is **undefined behaviour**.
> But will hence result in MUCH smaller binaries.



## auto nightly
natrix will auto detect when its compiled on nightly and use certain (non-public-facing) features. this is one of the reasons its recommended to use nightly rust.
* optimize text updates, on stable updating a text node is done via `replace_child`, on nightly it uses `set_text_content`
