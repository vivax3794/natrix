# Assets

> [!IMPORTANT]
> Similar to CSS, assets do not work without the natrix cli as a bundler.

Assets can be bundled with the `asset!` macro, it will include the given file path (relative to the crates `Cargo.toml`), the macro expands to the runtime path of the asset (prefixed with `/`, or `base_path` if set)

```rust,ignore
# extern crate natrix;
# use natrix::prelude::*;
# let _: e::HtmlElement<(), _> =
e::img()
    .src(natrix::asset!("./assets/my_img.png"))
# ;
```

This will include `/path/to/crate/./assets/my_img.png` in the `dist` folder, and expand to something like this in rust:

```rust,no_run
# extern crate natrix;
# use natrix::prelude::*;
# let _: e::HtmlElement<(), _> =
e::img()
    .src("/SOME_HASH-my_img.png")
# ;
```

> [!TIP]
> The dev server actually serves the assets from their source paths, so you dont have to worry about the files being copied on every reload.
