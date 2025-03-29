alias c := check
alias p := publish

test: && integration_tests
    cargo +nightly nextest run --all-features --workspace --exclude "integration_tests"

    cd natrix && rustup run stable wasm-pack test --headless --chrome
    cd natrix && rustup run nightly wasm-pack test --headless --chrome --all-features

check: test
    cargo +stable hack clippy --feature-powerset --skip nightly --tests -- -Dwarnings
    cargo +nightly hack clippy --feature-powerset --tests -- -Dwarnings

    cd natrix_macros && cargo-bounds test
    cd natrix && cargo-bounds test

[working-directory: "./integration_tests"]
integration_tests:
    #!/usr/bin/bash
    set -e

    cleanup() {
        echo "Cleaning up..."
        if [ -n "$trunk_pid" ]; then
            kill "$trunk_pid" 2>/dev/null
        fi
        if [ -n "$chrome_pid" ]; then
            kill "$chrome_pid" 2>/dev/null
        fi
    }
    trap cleanup EXIT

    RUSTFLAGS="-Awarnings" trunk build
    RUSTFLAGS="-Awarnings" trunk serve --port 4444 --ignore .. &
    trunk_pid=$!

    chromedriver --port=9999 &
    chrome_pid=$!

    sleep 1
    cargo nextest run -j 1

# Publish the crate to crates.io
publish: fmt check
    cargo publish -p natrix_macros
    cargo publish -p natrix

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
