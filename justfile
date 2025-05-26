alias c := check
alias t := test
alias f := full

# Run the default set of quick tests
default:
    cargo +nightly nextest run --all-features -p natrix
    cd natrix && rustup run nightly wasm-pack test --headless --chrome --all-features

# Run the full set of tests and checks
full: test check check_docs check_deps

# Run the full set of tests
test: test_native test_web integration_tests_dev integration_tests_build project_gen_test

# Run tests that are not dependent on the web
test_native:
    cargo +nightly nextest run --all-features -p natrix
    cargo +nightly nextest run --all-features -p natrix --release

# Run tests that are dependent on the web
[working-directory: './natrix']
test_web:
    rustup run stable wasm-pack test --headless --chrome --features test_utils
    rustup run nightly wasm-pack test --headless --chrome --all-features

# Run clippy on all packages and all features
check:
    cargo fmt --check

    cargo +stable clippy -p natrix-cli -- -Dwarnings

    cargo +stable hack clippy -p natrix --target wasm32-unknown-unknown --each-feature --skip nightly --tests -- -Dwarnings
    cargo +nightly hack clippy -p natrix --target wasm32-unknown-unknown --each-feature --tests -- -Dwarnings
    cargo +stable hack clippy -p natrix --target wasm32-unknown-unknown --each-feature --skip nightly --tests --release -- -Dwarnings
    cargo +nightly hack clippy -p natrix --target wasm32-unknown-unknown --each-feature --tests --release -- -Dwarnings

check_deps:
    cargo hack udeps --each-feature --ignore-private --all-targets
    cargo outdated -R --workspace --exit-code 1
    cd natrix && cargo deny check all --exclude-dev
    cd natrix-cli && cargo deny check all --exclude-dev --hide-inclusion-graph

# Check the documentation for all packages
# And for typos in the docs
check_docs: && check_book
    typos
    cargo test --doc --all-features --workspace 

check_book:
    cd docs && mdbook build
    rm -r target/debug/deps/*natrix*
    rm -r target/debug/deps/*wasm_bindgen_test*
    rustup run nightly cargo build -p natrix --all-features --tests
    cd docs && rustup run nightly mdbook test -L ../target/debug/deps

# Run the integration tests
# These will spawn the `natrix dev` server and run the tests against it
[working-directory: "./integration_tests"]
integration_tests_dev: install_cli
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
    natrix build --profile dev

    chromedriver --port=9999 &
    chrome_pid=$!

    (natrix dev --port 8000 > /dev/null 2>&1) &
    natrix_pid=$!
    cargo nextest run -j 1

    kill $natrix_pid 2>/dev/null || true
    (natrix dev --profile release --port 8000 > /dev/null 2>&1) & 
    natrix_pid=$!
    cargo nextest run -j 1

[working-directory: "./integration_tests"]
integration_tests_build: install_cli
    #!/usr/bin/bash
    set -e

    cleanup() {
        echo "Cleaning up..."
        if [ -n "$python_pid" ]; then
            kill "$python_pid" 2>/dev/null || true
        fi
        if [ -n "$chrome_pid" ]; then
            kill "$chrome_pid" 2>/dev/null || true
        fi
    }
    trap cleanup EXIT

    chromedriver --port=9999 &
    chrome_pid=$!

    (python3 -m http.server > /dev/null 2>&1) & # Yes we are not serving the dist dir, this is to test the BASE_PATH option
    python_pid=$!

    natrix build
    cargo nextest run -j 1 --features build_test

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
    cargo install --path natrix-cli --profile dev

# Open the guide book with a auto reloading server
[working-directory: './docs']
book: 
    mdbook serve --open

# Install the book dependencies
book_deps:
    command -v mdbook || cargo binstall -y mdbook
    command -v mdbook-callouts || cargo binstall -y mdbook-callouts
    command -v mdbook-rustdoc-link || cargo install mdbookkit --features rustdoc-link

# Install all dev tool dependencies
dev_deps: book_deps
    command -v typos || cargo binstall -y typos-cli
    command -v cargo-hack || cargo binstall -y cargo-hack
    command -v cargo-nextest || cargo binstall -y cargo-nextest
    command -v wasm-pack || cargo binstall -y wasm-pack
    command -v wasm-bindgen || cargo binstall -y wasm-bindgen-cli
    command -v cargo-deny || cargo binstall -y cargo-deny
    command -v cargo-udeps || cargo binstall -y cargo-udeps
    command -v cargo-outdated || cargo binstall -y cargo-outdated

# Check for the presence of all required system dependencies
# That there is no cross-platform way to install
health_check:
    command -v chromedriver || (echo "chromedriver not found, required for integration tests" && exit 1)
    command -v wasm-opt || (echo "wasm-opt not found, required for integration tests" && exit 1)
    command -v cargo || (echo "Cargo not found, required for everything" && exit 1)
    command -v cargo-clippy || (echo "Clippy not found, required for linting" && exit 1)
    command -v rust-analyzer || (echo "Rust-Analyzer not found, required for book" && exit 1)
    command -v python3 || (echo "Python3 not found, required for integration tests" && exit 1)

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
    command -v cargo-deny || (echo "cargo-deny not found, required for security checks" && exit 1)
    command -v cargo-udeps || (echo "cargo-udeps not found, required for dependency checks" && exit 1)
    command -v cargo-outdated || (echo "cargo-outdated not found, required for dependency checks" && exit 1)

# Generate and open public docs
docs:
    cargo doc --open -p natrix --lib --all-features

# Remove all build artifacts
clean:
    cargo clean
    cd docs && mdbook clean
    rm -rv integration_tests/dist || true

gh_action:
    act -P ubuntu-latest=catthehacker/ubuntu:full-latest -W .github/workflows/run_tests.yml


[working-directory: './stress_test_binary_size']
stress_size: install_cli
    wc -c dist/code_bg.wasm
    natrix build
    wc -c dist/code_bg.wasm

[working-directory: './benchmark']
bench: install_cli
    #!/usr/bin/bash
    set -e
    cleanup() {
        echo "Cleaning up..."
        if [ -n "$python_pid" ]; then
            kill "$python_pid" 2>/dev/null || true
        fi
    }
    trap cleanup EXIT

    natrix build
    (cd dist && python3 -m http.server 8888 2>/dev/null) &
    python_pid=$!

    wasm_bench
