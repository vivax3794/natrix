alias c := check_small
alias f := check_full
alias p := publish

check_small:
    cargo +nightly nextest run --all-features
    cd natrix && RUSTFLAGS="-Awarnings" rustup run nightly wasm-pack test --headless --chrome --all-features
    cargo +nightly clippy --all-features 

check_full: test_full lint_full test_bounds

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
    RUSTFLAGS="-Awarnings" trunk serve --port 4444 &
    trunk_pid=$!

    chromedriver --port=9999 &
    chrome_pid=$!

    sleep 1
    cargo nextest run -j 1

test_bounds:
    cd natrix_macros && cargo-bounds test
    cd natrix && cargo-bounds test

# Publish the crate to crates.io
publish: fmt check_full
    cargo publish -p natrix_macros
    cargo publish -p natrix

mutation:
    RUSTFLAGS="--cfg=mutants -C codegen-units=1" cargo mutants --workspace --test-workspace true --jobs 4 -- --lib --all-features

test_full: && test_web_full integration_tests
    cargo +stable hack nextest run --each-feature --skip nightly --no-tests pass
    cargo +nightly hack nextest run --each-feature --no-tests pass
    cargo +nightly nextest run --release --all-features

[working-directory: "./natrix"]
test_web_full:
    #!/usr/bin/bash
    set -e

    while IFS= read -r line || [ -n "$line" ]; do
        modified_line=$(echo "$line" | sed 's/cargo/rustup run stable/g')
        echo "Executing: $modified_line 🎀"
        RUSTFLAGS="-Awarnings" eval "$modified_line"
    done < <(cargo hack wasm-pack test --headless --chrome --skip nightly --each-feature --features test_utils --print-command-list --no-manifest-path)

    while IFS= read -r line || [ -n "$line" ]; do
        modified_line=$(echo "$line" | sed 's/cargo/rustup run nightly/g')
        echo "Executing: $modified_line 🎀"
        RUSTFLAGS="-Awarnings" eval "$modified_line"
    done < <(cargo hack wasm-pack test --headless --chrome --each-feature --features test_utils --print-command-list --no-manifest-path)

    RUSTFLAGS="-Awarnings" rustup run nightly wasm-pack test --headless --chrome --all-features --release
    

lint_full:
    cargo +stable hack clippy --feature-powerset --skip nightly --tests -- -Dwarnings
    cargo +nightly hack clippy --feature-powerset --tests -- -Dwarnings
    cargo +nightly clippy --all-features --release

[working-directory: './bench_project']
bench:
    rustup run stable wasm_bench
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
    rm -rv mutants.out* || true
    rm -v bench_project/.tmp* || true
    rm -rv integration_tests/dist || true
