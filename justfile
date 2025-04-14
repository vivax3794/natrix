alias c := check
alias t := test
alias f := full

# Run the default set of quick tests
default: test_native test_web

# Run the full set of tests and checks
full: test check check_docs

# Run the full set of tests
test: test_native test_web integration_tests test_css_tree_shaking project_gen_test test_homepage

# Run tests that are not dependent on the web
test_native:
    cargo +nightly nextest run --all-features -p natrix

# Run tests that are dependent on the web
[working-directory: './natrix']
test_web:
    rustup run stable wasm-pack test --headless --chrome --features test_utils
    rustup run nightly wasm-pack test --headless --chrome --all-features

# Run the homepage tests
[working-directory: './homepage']
test_homepage:
    rustup run nightly wasm-pack test --headless --chrome


# Run clippy on all packages and all features
check:
    cargo fmt --check

    cargo +stable clippy -p natrix_macros -- -Dwarnings
    cargo +stable clippy -p natrix_shared -- -Dwarnings
    cargo +stable clippy -p natrix-cli -- -Dwarnings

    cargo +stable hack clippy -p natrix --target wasm32-unknown-unknown --each-feature --skip nightly --tests -- -Dwarnings
    cargo +nightly clippy -p natrix --target wasm32-unknown-unknown --all-features --tests -- -Dwarnings

    cargo +nightly clippy -p homepage -- -Dwarnings


# Check the documentation for all packages
# And for typos in the docs
check_docs:
    typos
    cargo test --doc --all-features --workspace 

    cd docs && mdbook build
    rm -r target/debug/deps/*natrix*
    rm -r target/debug/deps/*wasm_bindgen_test*
    cargo build -p natrix --all-features --tests
    CARGO_PKG_NAME="mdbook_example" cd docs && mdbook test -L ../target/debug/deps

# Run the integration tests
# These will spawn the `natrix dev` server and run the tests against it
[working-directory: "./integration_tests"]
integration_tests: install_cli
    #!/usr/bin/bash
    set -e

    cleanup() {
        echo "Cleaning up..."
        if [ -n "$natrix_pid" ]; then
            kill "$natrix_pid" 2>/dev/null || true
        fi
        if [ -n "$chrome_pid" ]; then
            kill "$chrome_pid" 2>/dev/null || true
        fi
    }
    trap cleanup EXIT

    natrix build
    natrix build -p dev

    chromedriver --port=9999 &
    chrome_pid=$!
    sleep 1

    (natrix dev > /dev/null 2>&1) &
    natrix_pid=$!
    sleep 1

    cargo nextest run -j 1

    kill $natrix_pid 2>/dev/null || true
    (natrix dev -p release > /dev/null 2>&1) &
    natrix_pid=$!
    sleep 1

    cargo nextest run -j 1

# Check that css tree-shaking works
[working-directory: "./integration_tests"]
test_css_tree_shaking: install_cli
    natrix build -p dev
    grep "I_amNotUsed" dist/styles.css

    natrix build -p release
    ! grep "I_amNotUsed" dist/styles.css




# Run the project generation tests
# These will use `natrix new` to create a new project and then build it
[working-directory: "/tmp"]
project_gen_test: install_cli
    NATRIX_PATH="{{justfile_directory()}}/natrix" natrix new test_project --stable
    cd test_project && rustup run stable natrix build
    cd test_project && rustup run stable wasm-pack test --headless --chrome

    NATRIX_PATH="{{justfile_directory()}}/natrix" natrix new test_project
    cd test_project && rustup run nightly natrix build
    cd test_project && rustup run nightly wasm-pack test --headless --chrome

# Install the CLI for use in tests
# This installs it in debug mode and should *not* be used for actually installing
install_cli:
    cargo install --path natrix-cli --profile dev --frozen

# Open the guide book with a auto reloading server
[working-directory: './docs']
book: 
    mdbook serve --open

# Serve the natrix home page with a auto reloading server
[working-directory: './homepage']
homepage: install_cli
    natrix dev

# Install the book dependencies
book_deps:
    command -v mdbook || cargo binstall mdbook
    command -v mdbook-callouts || cargo binstall mdbook-callouts
    command -v mdbook-rustdoc-link || cargo install mdbookkit --features rustdoc-link

# Install all dev tool dependencies
dev_deps: book_deps
    command -v typos || cargo binstall typos-cli
    command -v cargo-hack || cargo binstall cargo-hack
    command -v cargo-nextest || cargo binstall cargo-nextest
    command -v wasm-pack || cargo binstall wasm-pack

# Check for the presence of all required system dependencies
# That there is no cross-platform way to install
health_check:
    command -v chromedriver || (echo "chromedriver not found, required for integration tests" && exit 1)
    command -v wasm-opt || (echo "wasm-opt not found, required for integration tests" && exit 1)

    # These do have cross-platform ways to install
    # But we check for them here to make sure they are installed
    command -v wasm-pack || (echo "wasm-pack not found, required for unit tests" && exit 1)
    command -v cargo-nextest || (echo "cargo-nextest not found, required for testing" && exit 1)
    command -v cargo-hack || (echo "cargo-hack not found, required for linting" && exit 1)
    command -v typos || (echo "cargo-typos not found, required for linting" && exit 1)
    command -v mdbook || (echo "mdbook not found, required for documentation" && exit 1)
    command -v mdbook-callouts || (echo "mdbook-callouts not found, required for documentation" && exit 1)
    command -v mdbook-rustdoc-link || (echo "mdbook-rustdoc-link not found, required for documentation" && exit 1)
    command -v wasm-bindgen || (echo "wasm-bindgen not found, required for building wasm" && exit 1)

# Generate and open public docs
docs:
    cargo doc --open -p natrix --lib --all-features

# Remove all build artifacts
clean:
    cargo clean
    cd docs && mdbook clean
    rm -rv integration_tests/dist || true
