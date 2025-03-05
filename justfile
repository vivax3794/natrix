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

test_full: && test_web_full
    cargo +stable hack nextest run --feature-powerset --skip nightly --no-tests warn
    cargo +nightly hack nextest run --feature-powerset --features nightly --ignore-unknown-features --no-tests warn

test_small: && test_web_small
    cargo +nightly hack nextest run --each-feature --no-tests warn

[working-directory: "./natrix"]
test_web_full:
    #!/usr/bin/bash
    set -e
    while IFS= read -r line || [ -n "$line" ]; do
        modified_line=$(echo "$line" | sed 's/cargo/rustup run stable/g')
        echo "Executing: $modified_line 🎀"
        eval "$modified_line"
    done < <(cargo hack wasm-pack test --headless --chrome --feature-powerset --skip nightly --print-command-list --no-manifest-path)

    #!/usr/bin/bash
    while IFS= read -r line || [ -n "$line" ]; do
        modified_line=$(echo "$line" | sed 's/cargo/rustup run nightly/g')
        echo "Executing: $modified_line 🎀"
        eval "$modified_line"
    done < <(cargo hack wasm-pack test --headless --chrome --feature-powerset --features nightly --print-command-list --no-manifest-path)

[working-directory: "./natrix"]
test_web_small:
    #!/usr/bin/bash
    set -e
    while IFS= read -r line || [ -n "$line" ]; do
        modified_line=$(echo "$line" | sed 's/cargo/rustup run nightly/g')
        echo "Executing: $modified_line 🎀"
        eval "$modified_line"
    done < <(cargo hack wasm-pack test --headless --chrome --each-feature --print-command-list --no-manifest-path)


lint_full:
    cargo +stable hack clippy --feature-powerset --skip nightly -- -Dwarnings
    cargo +nightly hack clippy --feature-powerset -- -Dwarnings

lint_small:
    cargo +nightly hack clippy --each-feature -- -Dwarnings

fmt:
    cargo fmt

# Open the guide book with a auto reloading server
[working-directory: './docs']
book:
    mdbook serve --open

# Generate and open public docs
docs:
    cargo doc --open -p natrix --lib

# Generate and open docs for internal items
docs_internal:
    cargo doc --open -p natrix --lib --document-private-items



# Remove all build artifacts
clean:
    cargo clean
    rm -rv docs/book || true
