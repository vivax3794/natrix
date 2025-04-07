alias c := check
alias p := publish

test: && integration_tests project_gen_test
    cargo +nightly nextest run --all-features --workspace --exclude "integration_tests"

    cd natrix && rustup run stable wasm-pack test --headless --chrome
    cd natrix && rustup run nightly wasm-pack test --headless --chrome --all-features

check:
    cargo fmt --check
    cargo +stable hack clippy --feature-powerset --skip nightly --tests -- -Dwarnings
    cargo +nightly hack clippy --feature-powerset --tests -- -Dwarnings

bounds:
    cd natrix_macros && cargo-bounds test
    cd natrix && cargo-bounds test

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
    natrix new test_project
    cd test_project && cargo check --all-features

install_cli:
    cargo install --path natrix-cli

# Publish the crate to crates.io
publish: fmt check
    cargo publish -p natrix_shared
    cargo publish -p natrix_macros
    cargo publish -p natrix
    cargo publish -p natrix-cli

[working-directory: './bench_project']
bench:
    rustup run nightly wasm_bench

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
    rm -v bench_project/.tmp* || true
    rm -rv integration_tests/dist || true
