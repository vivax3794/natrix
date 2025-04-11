# FAQ

## Why is there no `html!` macro?
Natrix does not use a macro DSL for HTML generation. This is to avoid the issues that come with macro-based DSLs, such as breaking formatting and Rust Analyzer support. Instead, Natrix uses a builder pattern to create HTML elements, which is more idiomatic in Rust and provides a smoother developer experience.

> [!QUESTION]
> Is this a feature you want? Consider making a crate for it!
> It should be fully possible to create a macro that generates the builder pattern calls.

## Why a custom build tool instead of using `Trunk`
Features such as natrixses unique [css bundling](css.md) require full control of the build process in a way that if we wanted to stick with Trunk would still require us to have a custom tool calling it.
By fully taking control of the build process natrix applies the best possible optimizations by default, for example minifying the css and js files, as well as removing dead code from the css.
As well as using various cargo and rust build flags to optimize the binary size.

In addition a interesting thing `natrix build` does is inspect the enabled features and change its compilation automatically, for example if the [`panic_hook` feature](features.md#panic_hook) is disabled it will compile with aggressive DCE that eliminates all panic code, including branches that lead to panics.

## Does natrix handle css for dependencies?
Yes, all css defined in dependencies is bunlded (and DCE-ed!) automatically if you use a dependency from crates.io with no extra setup needed. Similar to what you might be familiar with from component libraries in the js world. This is a unique feature of natrix, as most other rust frameworks require you to manually add the css to your project.

## Can you use `natrix` with other frameworks?
In theory you can, but certain features like css and asset bundling will not work as expected. But the core component system should work fine.
