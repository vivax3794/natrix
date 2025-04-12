alias c := check
alias t := test
alias p := publish
alias f := full

default: test_native test_web
full: test check check_docs

test: test_native test_web integration_tests project_gen_test

test_native:
    cargo +nightly nextest run --all-features -p natrix

[working-directory: './natrix']
test_web:
    rustup run stable wasm-pack test --headless --chrome --features test_utils
    rustup run nightly wasm-pack test --headless --chrome --all-features

check:
    cargo fmt --check

    cargo +stable clippy -p natrix_macros -- -Dwarnings
    cargo +stable clippy -p natrix_shared -- -Dwarnings
    cargo +stable clippy -p natrix-cli -- -Dwarnings

    cargo +stable hack clippy -p natrix --target wasm32-unknown-unknown --each-feature --skip nightly --tests -- -Dwarnings
    cargo +nightly clippy -p natrix --target wasm32-unknown-unknown --all-features --tests -- -Dwarnings


check_docs:
    typos
    cargo test --doc --all-features --workspace 

    cd docs && mdbook build
    rm -r target/debug/deps/*natrix*
    cargo build -p natrix --all-features
    cd docs && mdbook test -L ../target/debug/deps

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

    natrix build
    natrix build -p dev

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
    NATRIX_PATH="{{justfile_directory()}}/natrix" natrix new test_project --stable
    cd test_project && rustup run stable natrix build

    NATRIX_PATH="{{justfile_directory()}}/natrix" natrix new test_project
    cd test_project && rustup run nightly natrix build

install_cli:
    cargo install --path natrix-cli --profile dev --frozen

# Publish the crate to crates.io
publish: fmt full
    cargo publish -Z package-workspace -p natrix_shared -p natrix_macros -p natrix-cli -p natrix

fmt:
    cargo fmt

# Open the guide book with a auto reloading server
[working-directory: './docs']
book: 
    mdbook serve --open

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
