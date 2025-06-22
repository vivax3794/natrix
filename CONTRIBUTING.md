# Contributing to Natrix

Thank you for considering contributing to Natrix! This guide will help you get started with the development process.

## Project Structure

- **crates/natrix/**: Core framework library
- **crates/natrix_macros/**: Procedural macros for components and CSS
- **crates/natrix_shared/**: Shared utilities between core and CLI
- **crates/natrix-cli/**: CLI tool for project management
- **ci/**: End-to-end testing, benchmarks, etc
- **docs/**: User guide (mdBook)

## Development Setup

Natrixse includes a large test suit. <TODO>

## Development Workflow

<TODO>

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
> While the codebase curretnly uses this, once we hit a proper public release we plan to migrate to github issues.

* `NOTE` - Justifications or context for surrounding code 
* `HACK` - Working around a third party bug, link to relevant issues.
* `INVARIANT` - A *internal* invarant that must be upheld by calling code, these a good target for refactor to move invariants to compile time.

* `TEST` - A test that should get written
* `TODO` - Unimplemented planned feature
* `MAYBE` - Features we might or might not implement
* `BUG` - Something is broken
* `SPEC` - A un-enforced web-standard invariant, these are candidates for intruducing more compile-time or runtime-checks, but are not hard todos. They should be considerd a blend of `TODO`, `MAYBE`, and `BUG`

## Pull Request Process
1. If possible please try to run test suits before creating a PR. If you local machine takes too long the tests will always be run on CI anyway.
2. Update documentation if necessary
3. Add tests for new functionality

## Documentation

When adding features, please:
1. Update the mdBook in the `docs/` directory
2. Add rustdoc comments to public APIs
