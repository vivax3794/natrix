# Contributing to Natrix

Thank you for considering contributing to Natrix! This guide will help you get started with the development process.

## Project Structure

- **crates/natrix/**: Core framework library
- **crates/natrix_macros/**: Procedural macros for state and assets
- **crates/natrix_shared/**: Shared utilities between core and CLI
- **crates/natrix-cli/**: CLI tool for project management
- **ci/**: End-to-end testing, benchmarks, etc
- **docs/**: User guide (mdBook)

## Development Setup

Natrix uses [Dagger](https://dagger.io/) for containerized CI/CD pipeline execution and [`just`](https://github.com/casey/just) for task automation. Please follow the [Dagger installation guide](https://docs.dagger.io/install) to set up your development environment.

**Why Dagger?** We use Dagger because Natrix has extensive E2E tests and benchmarks that would otherwise require installing around 10 different tools (Chrome, wasm-bindgen, wasm-opt, various Rust tools, etc.). With Dagger, all dependencies are containerized, ensuring consistent builds across environments.

## Development Workflow

### Running the CI Pipeline

The primary way to run tests is through the justfile targets:

- **`just full [jobs]`** - Run the complete test suite (same as CI)
- **`just quick [jobs]`** - Run the "Quick" test subset
- **`just fix`** - Runs automatic fixing of typos, formatting, and outdated snapshot tests.

> [!IMPORTANT]
> * The snapshot tests will assume the new results are correct, always verify the new output is correct in the error logs first.
> * `typos` might assume the wrong correction, always inspect the suggestions in its errors before running `just fix`

Examples:
```bash
just full        # Run full tests with 1 job
just full 4      # Run full tests with 4 jobs
just quick       # Run quick tests with 1 job  
just quick 8     # Run quick tests with 8 jobs
```

### Test Categories

The **Quick** tests are designed to run quickly and catch 90% of issues you might introduce while working on the project. These include unit tests, linting, formatting, and basic checks. All other tests (like integration tests, dependency checks, and cross-toolchain validation) will run in the CI anyway.

The **Full** test suite includes all quick tests plus comprehensive integration tests, dependency analysis, and multi-toolchain validation.

### Quick Development Iteration

For quick iteration during development, you can also use standard Rust commands like `cargo clippy` and `cargo test`. However, running `just quick` ensures you catch most issues before pushing, and the full CI pipeline ensures reproducible builds.

## Code Style
Natrix uses a wide range of clippy lints. But we do often use `#[expect]` on certain areas.
**But, this should be done sparingly.**

In most cases make use of `log_or_panic_*` macros to panic on debug builds, but only log error in production. Allowing production panics should only be done in extremely specific cases, effectively only when said panic would also **instantly** be hit in debug builds, for example problems mounting the root element.

Additionally natrix has important invariants in terms of its reactivity system that must not be invalidated.
When implementing new features, try to build on existing functionality in order to minimize the risk of breaking these invariants.

## Comment style
Natrix has a few "comment tags" we use:

* `NOTE` - Justifications or context for surrounding code 
* `HACK` - Working around a third party bug, link to relevant issues.
* `INVARIANT` - A *internal* invariant that must be upheld by calling code, these a good target for refactor to move invariants to compile time.

## Pull Request Process
1. If possible please try to run test suits before creating a PR. If you local machine takes too long the tests will always be run on CI anyway.
2. Update documentation if necessary
3. Add tests for new functionality

## Documentation

When adding features, please:
1. Update the mdBook in the `docs/` directory
2. Add rustdoc comments to public APIs
