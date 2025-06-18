# Introduction 

<img src="https://github.com/vivax3794/natrix/raw/master/assets/logo.png" alt="Logo" width="300" height="300">

Natrix is a ***Rust-first*** frontend framework. Embracing Rust’s strengths—leveraging smart pointers, derive macros, the builder pattern, and other idiomatic Rust features to create a truly native experience.

# A Simple Example

A simple counter in Natrix looks like this:

```rust,no_run
# extern crate natrix;
# use natrix::prelude::*;
#
#[derive(Component)]
struct Counter(usize);

impl Component for Counter {
    fn render() -> impl Element<Self> {
        e::button()
            .text(|ctx: R<Self>| *ctx.0)
            .on::<events::Click>(|ctx: E<Self>, _, _| {
                *ctx.0 += 1;
            })
    }
}
#
# fn main() {
#    natrix::mount(Counter(0));
# }
```

## Standout features
* ✅ **No macro DSL** – Macro-based DSLs break formatting & Rust Analyzer support. Natrix avoids them completely for a smoother dev experience.
* ✅ **Derive macros for reactive state** – No need for `useSignal` everywhere, define a struct, thats it.
* ✅ **Callbacks use references to state** – Instead of closures capturing state setters, Natrix callbacks take a reference to the state, which better aligns with Rust’s ownership model.
* ✅ **JS style bundling solution** – Natrix has a compile time css and asset bundling solution that works with dependencies out of the box.

# Design Goals
* **Developer experience first** – Natrix is designed to feel natural for Rust developers.
* **Idiomatic Rust** – We use Rust-native features & patterns, not what worked for js.
* **Stop porting JS to Rust** – Rust is an amazing language, let’s build a frontend framework that actually feels like Rust.

