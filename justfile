alias c := check
alias t := test
alias p := publish
alias f := full

default: test_native test_web
full: test check

test: test_native test_web integration_tests project_gen_test

test_native:
    cargo +nightly nextest run --all-features --workspace --exclude "integration_tests"

[working-directory: './natrix']
test_web:
    rustup run stable wasm-pack test --headless --chrome --features test_utils
    rustup run nightly wasm-pack test --headless --chrome --all-features

check:
    cargo fmt --check
    cargo +stable hack clippy --each-feature --skip nightly --tests -- -Dwarnings
    cargo +nightly hack clippy --each-feature --tests -- -Dwarnings

[working-directory: "./integration_tests"]
integration_tests: install_cli
    #!/usr/bin/bash
    set -e

    cleanup() {
        echo "Cleaning up..."
        if [ -n "$natrix_pid" ]; then
            kill "$natrix_pid" 2>/dev/null
        fi
        if [ -n "$chrome_pid" ]; then
            kill "$chrome_pid" 2>/dev/null
        fi
    }
    trap cleanup EXIT

    chromedriver --port=9999 &
    chrome_pid=$!
    sleep 1

    natrix dev &
    natrix_pid=$!
    sleep 1

    cargo nextest run -j 1

    kill $natrix_pid
    natrix dev -p release &
    natrix_pid=$!
    sleep 1

    cargo nextest run -j 1

[working-directory: "/tmp"]
project_gen_test: install_cli
    rm -rf ./test_project || true
    NATRIX_PATH="{{justfile_directory()}}/natrix" natrix new test_project --stable
    cd test_project && rustup run stable natrix build

    rm -rf ./test_project || true
    NATRIX_PATH="{{justfile_directory()}}/natrix" natrix new test_project
    cd test_project && rustup run nightly natrix build

install_cli:
    cargo install --path natrix-cli --profile dev --frozen

# Publish the crate to crates.io
publish: fmt test check
    cargo publish -Z package-workspace -p natrix_shared -p natrix_macros -p natrix-cli -p natrix

fmt:
    cargo fmt

# Open the guide book with a auto reloading server
[working-directory: './docs']
book:
    mdbook serve --open

# Generate and open public docs
docs:
    cargo doc --open -p natrix --lib --all-features

# Remove all build artifacts
clean:
    cargo clean
    rm -rv docs/book || true
    rm -rv integration_tests/dist || true
