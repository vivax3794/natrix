# Contributing to Natrix

Thank you for considering contributing to Natrix! This guide will help you get started with the development process.

## Project Structure

- **natrix/**: Core framework library
- **natrix_macros/**: Procedural macros for components and CSS
- **natrix_shared/**: Shared utilities between core and CLI
- **natrix-cli/**: CLI tool for project management
- **homepage/**: Natrix website/documentation
- **integration_tests/**: End-to-end testing
- **docs/**: User guide (mdBook)

## Development Setup

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
- Native tests: `just test_native`
- Web tests: `just test_web`
- Integration tests: `just integration_tests_dev`

### Code Quality

- Linting: `just check` (alias: `just c`)
- Documentation checks: `just check_docs`
- Full suite: `just full` (alias: `just f`)

### Documentation

- Serve docs locally: `just book`
- API documentation: `just docs`

### Development

- Run homepage: `just homepage`
- Clean artifacts: `just clean`

## Pull Request Process

1. Ensure your code passes all tests and checks (`just full`)
2. Update documentation if necessary
3. Add tests for new functionality

## Testing

Natrix has several test categories:
- Native Rust tests
- Web-specific tests using wasm-pack
- Integration tests with WebDriver
- Documentation tests

## Documentation

When adding features, please:
1. Update the mdBook in the `docs/` directory
2. Add rustdoc comments to public APIs
