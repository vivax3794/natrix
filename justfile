alias c := check
alias t := test
alias p := publish
alias f := full

# Run the default set of quick tests
default: test_native test_web

# Run the full set of tests and checks
full: test check check_docs

# Run the full set of tests
test: test_native test_web integration_tests project_gen_test

# Run tests that are not dependent on the web
test_native:
    cargo +nightly nextest run --all-features -p natrix

# Run tests that are dependent on the web
[working-directory: './natrix']
test_web:
    rustup run stable wasm-pack test --headless --chrome --features test_utils
    rustup run nightly wasm-pack test --headless --chrome --all-features

# Run clippy on all packages and all features
check:
    cargo fmt --check

    cargo +stable clippy -p natrix_macros -- -Dwarnings
    cargo +stable clippy -p natrix_shared -- -Dwarnings
    cargo +stable clippy -p natrix-cli -- -Dwarnings

    cargo +stable hack clippy -p natrix --target wasm32-unknown-unknown --each-feature --skip nightly --tests -- -Dwarnings
    cargo +nightly clippy -p natrix --target wasm32-unknown-unknown --all-features --tests -- -Dwarnings


# Check the documentation for all packages
# And for typos in the docs
check_docs:
    typos
    cargo test --doc --all-features --workspace 

    cd docs && mdbook build
    rm -r target/debug/deps/*natrix*
    cargo build -p natrix --all-features
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

# Run the project generation tests
# These will use `natrix new` to create a new project and then build it
[working-directory: "/tmp"]
project_gen_test: install_cli
    NATRIX_PATH="{{justfile_directory()}}/natrix" natrix new test_project --stable
    cd test_project && rustup run stable natrix build

    NATRIX_PATH="{{justfile_directory()}}/natrix" natrix new test_project
    cd test_project && rustup run nightly natrix build

# Install the CLI for use in tests
# This installs it in debug mode and should *not* be used for actually installing
install_cli:
    cargo install --path natrix-cli --profile dev --frozen

# Publish the crate to crates.io
publish: full
    cargo publish -Z package-workspace -p natrix_shared -p natrix_macros -p natrix-cli -p natrix

# Open the guide book with a auto reloading server
[working-directory: './docs']
book: 
    mdbook serve --open

# Install the book dependencies
book_deps:
    cargo install mdbook 
    cargo install mdbook-callouts
    cargo install mdbookkit --features rustdoc-link


# Generate and open public docs
docs:
    cargo doc --open -p natrix --lib --all-features

# Remove all build artifacts
clean:
    cargo clean
    cd docs && mdbook clean
    rm -rv integration_tests/dist || true
