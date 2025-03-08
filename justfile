alias t := test_small
alias c := check_small
alias l := lint_small
alias f := check_full
alias p := publish

check_small: test_small lint_small
check_full: test_full lint_full

# Publish the crate to crates.io
publish: fmt check_full
    cargo publish -p natrix_macros
    cargo publish -p natrix

mutation:
    RUSTFLAGS="--cfg=mutants -C codegen-units=1" cargo mutants --workspace --test-workspace true --jobs 4 -- --lib --all-features

test_full: && test_web_full
    cargo +stable hack nextest run --feature-powerset --skip nightly --no-tests warn
    cargo +nightly hack nextest run --feature-powerset --no-tests warn

test_small: && test_web_small
    cargo +nightly nextest run --all-features

[working-directory: "./natrix"]
test_web_full:
    #!/usr/bin/bash
    set -e
    # Firefox is ungodly slow so we only do one run with all features
    # Which should realisticly catch any bugs
    # (We run the full feature matrix on chrome and native tests because they are much faster to do so, so might as well)
    rustup run nightly wasm-pack test --headless --firefox --all-features

    set -e
    while IFS= read -r line || [ -n "$line" ]; do
        modified_line=$(echo "$line" | sed 's/cargo/rustup run stable/g')
        echo "Executing: $modified_line 🎀"
        eval "$modified_line"
    done < <(cargo hack wasm-pack test --headless --chrome --feature-powerset --skip nightly --print-command-list --no-manifest-path)

    while IFS= read -r line || [ -n "$line" ]; do
        modified_line=$(echo "$line" | sed 's/cargo/rustup run nightly/g')
        echo "Executing: $modified_line 🎀"
        eval "$modified_line"
    done < <(cargo hack wasm-pack test --headless --chrome --feature-powerset --print-command-list --no-manifest-path)
    

[working-directory: "./natrix"]
test_web_small:
    rustup run nightly wasm-pack test --headless --chrome --all-features

lint_full:
    cargo +stable hack clippy --feature-powerset --skip nightly --tests -- -Dwarnings
    cargo +nightly hack clippy --feature-powerset --tests -- -Dwarnings

lint_small:
    cargo +nightly clippy --all-features 

fmt:
    cargo fmt

# Open the guide book with a auto reloading server
[working-directory: './docs']
book:
    mdbook serve --open

# Generate and open public docs
docs:
    cargo doc --open -p natrix --lib --all-features

# Generate and open docs for internal items
docs_internal:
    cargo doc --open -p natrix --lib --all-features --document-private-items

# Remove all build artifacts
clean:
    cargo clean
    rm -rv docs/book || true
    rm -rv mutants.out* || true
