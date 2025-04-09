# Features

## `nightly`
> The framework contains internal nightly optimizations which are automatically used even without this feature flag. If you find a bug that only manifests on nightly (with the same feature flags) please report it.

Enables public features, which currently are:
### `must_not_suspend`
Annotates specific types with [`#[must_not_suspend]`](https://github.com/rust-lang/rust/issues/83310) which allow the compiler to warn you if you pass it across a await point. For this lint to take affect your code also needs to enable the feature (if you used the project template and selected nightly this is done already).
```rust
#![feature(must_not_suspend)]
#![forbid(must_not_suspend)] // can make `warn` if you want
```
This will catch issues like this in `use_async` contexts.
```rust
let borrow = ctx.borrow_mut().unwrap();
foo().await;
let x = borrow; // ERROR
```

### Default associated types
On nightly `EmitMessage` and `RecvMessage` are `Never` by default, which means they dont have to be defined;

## `async` (Default)
Enables the `ctx.use_async()` method, see the [Async](TODO) chapther for more information.

## `ergonomic_ops`
Allows using inplace operations (`+=`, `-=`, etc) as well as comparissons (`==`, `>`) on signals without dereferencing them.
```rust
|ctx: &S<Self>| {
    if ctx.value > 10 {
        ctx.value += 2;
    }
}
```
This is off by default as it is inconsistent with builtin smart pointers such as `Box` and `Rc`, and also can not get rid of the dereference in all possible situations. for example the following cases still need an explicit `*`.
```rust
*ctx.value = 10;
let foo = *ctx.value + 10;
```
