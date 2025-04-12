# Getting Started

## Installation

The natrix cli requires the following dependencies:

- [Rust/Cargo](https://www.rust-lang.org/)
- [wasm-bindgen](https://crates.io/crates/wasm-bindgen-cli)
- [wasm-opt](https://github.com/WebAssembly/binaryen), Usually installed via `binaryen` for your platform.

Install the natrix cli with the following command:

```bash
cargo install --locked natrix-cli
```

## Creating a new project

To create a new natrix project, run the following command:

```bash
natrix new <project-name>
```

This will by default use nightly rust, if you wish to use stable rust, you can use the `--stable` flag:

```bash
natrix new <project-name> --stable
```

All features work on stable rust, but nightly includes more optimizations as well as results in smaller binaries. As well as provides some quality of life improvements. see [Features](features.md) for more information.

## Running the project

To run the project, navigate to the project directory and run:

```bash
natrix dev
```

This will start a local server that auto reloads on changes. Try changing the text in `src/main.rs` to see the changes live.

## Further Reading

- [Components](components.md) - Components are the core of natrix, and are the most important part of the framework.
- [Html](html.md) - This goes over the [`html_elements`] module and how to use it.
- [Element](elements.md) - This goes over the [`Element`](element::Element) trait and which std types implement it.
