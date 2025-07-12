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

All features work on stable rust, but nightly includes more optimizations as well as results in smaller binaries. 

## Running the project

To run the project, navigate to the project directory and run:

```bash
natrix dev
```

This will start a local server that auto reloads on changes. Try changing the text in `src/main.rs` to see the changes live.
