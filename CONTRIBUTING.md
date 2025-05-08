# Contributing to Natrix

Thank you for considering contributing to Natrix! This guide will help you get started with the development process.

## Project Structure

- **natrix/**: Core framework library
- **natrix_macros/**: Procedural macros for components and CSS
- **natrix_shared/**: Shared utilities between core and CLI
- **natrix-cli/**: CLI tool for project management
- **integration_tests/**: End-to-end testing
- **docs/**: User guide (mdBook)

## Development Setup

Natrixse includes a large test suit using a multitude of tools. Below is a quick guilde to installing all of them.
But depending on the modifications you want to do, it might be better to instead just run the affected tests.
And see what you need to install.

1. Install Rust toolchains:
   ```sh
   rustup install nightly
   rustup default nightly
   ```

2. Install development dependencies:
   ```sh
   just dev_deps
   ```

3. Install system dependencies:
   - chromedriver
   - wasm-opt
   - rust-analyzer
   - python3

4. Verify your setup:
   ```sh
   just health_check
   ```

## Development Workflow

Natrix uses [just](https://github.com/casey/just) for running tasks:

### Testing

- Basic tests: `just default` (alias: `just`)
- All tests: `just test` (alias: `just t`)
- All checks: `just full` (alias: `just f`)

### Code Quality

- Linting: `just check`
- Documentation checks: `just check_docs`

### Documentation

- Serve docs locally: `just book`
- API documentation: `just docs`

### Development
- Clean artifacts: `just clean`

## Code Style
Natrix uses a wide range of clippy lints. But we do often use `#[expect]` on certain areas.
**But, this should be done sparingly.**

In most cases make use of `debug_expect` and `debug_panic` macros to panic on debug builds, but silently fail in production. Allowing production panics should only be done in extremely specific cases, effectively only when said panic would also **instantly** be hit in debug builds, for example problems mounting the root component.

Additionally natrix has important invariants in terms of its reactivity system that must not be invalidated.
When implementing new features, try to build on existing functionality in order to minimize the risk of breaking these invariants.

Natrix has two nightly "feature flags", `cfg(nightly)` and `cfg(feature = "nightly"`). Its important to understand their different usecases, `cfg(nightly)` is set automatically on nightly, and should be used for non-public facing optimizations. `cfg(feature = "nightly")` is set by the user, and should be used for public facing features that are only available on nightly.

## Pull Request Process
1. If possible please try to run affected test suits before creating a PR. If you local machine takes too long the tests will always be run on CI anyway.
2. Update documentation if necessary
3. Add tests for new functionality

## Testing
This will outline which tests get affected by which changes.

> [!IMPORTANT]
> This is not a exhaustive list. It is just a general recommendation if you are unfamiliar with the code base.
> Modifying these components might cause failures in other tests not listed.
> As a example, breaking `.class` in `natrix` will cause integration tests to fail.

- **natrix/** - `test_native`, `test_web`
- **natrix_macros/** - `integration_tests_dev`, `integration_tests_build`
- **natrix-cli/** - `integration_tests_dev`, `integration_tests_build`, `project_gen_test`
- **docs/** - `check_docs`
- **new dependency** - `check_deps`

additionally most changes will require `check` and `check_docs`.

## Documentation

When adding features, please:
1. Update the mdBook in the `docs/` directory
2. Add rustdoc comments to public APIs
