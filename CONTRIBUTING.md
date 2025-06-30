# Contributing to Natrix

Thank you for considering contributing to Natrix! This guide will help you get started with the development process.

## Project Structure

- **crates/natrix/**: Core framework library
- **crates/natrix_macros/**: Procedural macros for components and assets
- **crates/natrix_shared/**: Shared utilities between core and CLI
- **crates/natrix-cli/**: CLI tool for project management
- **ci/**: End-to-end testing, benchmarks, etc
- **docs/**: User guide (mdBook)

## Development Setup

Natrix uses [`just`](https://github.com/casey/just) for task automation and [`Earthly`](https://earthly.dev/) for containerized testing.

**Why Earthly?** We use Earthly because Natrix has extensive E2E tests and benchmarks that would otherwise require installing around 10 different tools (Python, Chrome, wasm-bindgen, wasm-opt, various Rust tools, etc.). With Earthly, all dependencies are containerized, ensuring consistent builds across environments.

**Okay why justfile as well then?** Because even the root earthly file contains "internal" targets, and justfile lets us easier expose "public" targets.
In addition justfile allow us to also define targets that run on host, such as `update_snapshot`, and more importantly the benchmarks run on the host to avoid docker overhead. 

## Development Workflow

### Available `just` Targets

Run `just --list` to see all available commands:

#### Testing & Quality
- **`just all`** - Run the complete test suite (same as CI)
- **`just core`** - Run core framework tests
- **`just update_snapshot`** - Update test snapshots using cargo-insta
- **`just fix_typos`** - Fix typos in the codebase using typos

#### Documentation
- **`just docs`** - Generate and open Rust documentation
- **`just book`** - Build and serve the mdBook documentation

#### Development Tools
- **`just install_cli`** - Install the natrix CLI tool locally (mainly for benchmarks and debugging)
- **`just stress_size`** - Run binary size stress tests on WASM output
- **`just bench`** - Run performance benchmarks (runs on host, not in container)

Most targets use Earthly to ensure reproducible builds. For quick iteration during development, you can also use standard Rust commands like `cargo clippy` and `cargo test`.

## Code Style
Natrix uses a wide range of clippy lints. But we do often use `#[expect]` on certain areas.
**But, this should be done sparingly.**

In most cases make use of `log_or_panic_*` macros to panic on debug builds, but only log error in production. Allowing production panics should only be done in extremely specific cases, effectively only when said panic would also **instantly** be hit in debug builds, for example problems mounting the root component.

Additionally natrix has important invariants in terms of its reactivity system that must not be invalidated.
When implementing new features, try to build on existing functionality in order to minimize the risk of breaking these invariants.

Natrix has two nightly "feature flags", `cfg(nightly)` and `cfg(feature = "nightly"`). Its important to understand their different usecases, `cfg(nightly)` is set automatically on nightly, and should be used for non-public facing optimizations. `cfg(feature = "nightly")` is set by the user, and should be used for public facing features that are only available on nightly.

## Comment style
Natrix has a few "comment tags" we use:

> [!NOTE]
> While the codebase currently uses this, once we hit a proper public release we plan to migrate to github issues.

* `NOTE` - Justifications or context for surrounding code 
* `HACK` - Working around a third party bug, link to relevant issues.
* `INVARIANT` - A *internal* invariant that must be upheld by calling code, these a good target for refactor to move invariants to compile time.

* `TEST` - A test that should get written
* `TODO` - Unimplemented planned feature
* `MAYBE` - Features we might or might not implement
* `BUG` - Something is broken

* `REFACTOR` - A fully working feature that could be improved.
* `PERF` - Potential for optimization
* `SPEC` - A un-enforced web-standard invariant, these are candidates for introducing more compile-time or runtime-checks, but are not hard todos. They should be considered a blend of `TODO`, `MAYBE`, and `BUG`

### Editor Setup for Comment Tags

Most editors have plugins that can highlight and navigate these comment tags. Here are some common setups:

**Neovim (todo-comments.nvim):**
```lua
{
    "folke/todo-comments.nvim",
    dependencies = { "nvim-lua/plenary.nvim" },
    opts = {
        keywords = {
            REFACTOR = { icon="󰃣" },
            MAYBE = { icon="?" },
            INVARIANT = { icon = " ", color = "warning" },
            SPEC = {
                icon=" ",
                color = "error",
            },
        }
    },
}
```

**VS Code:** Install the "Todo Tree" extension and add the custom keywords to your settings.json:
```json
{
    "todo-tree.general.tags": [
        "NOTE", "HACK", "INVARIANT", "TEST", "TODO", "MAYBE", "BUG", "REFACTOR", "PERF", "SPEC"
    ]
}
```

## Pull Request Process
1. If possible please try to run test suits before creating a PR. If you local machine takes too long the tests will always be run on CI anyway.
2. Update documentation if necessary
3. Add tests for new functionality

## Documentation

When adding features, please:
1. Update the mdBook in the `docs/` directory
2. Add rustdoc comments to public APIs
