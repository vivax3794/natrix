alias t := test
alias c := check
alias p := publish
alias l := lint

default: check

# Publish the crate to crates.io
publish: check
    cargo publish -p natrix_macros
    cargo publish -p natrix

# Lint and run tests
check: test lint

test:
    just test_both "--"
    just test_both "--no-default-features"
    just test_both "'--features ergonomic_ops'"
    just test_setup nightly "--features nightly"
    just test_setup nightly "--all-features"

test_both f:
    just test_setup stable {{f}}
    just test_setup nightly {{f}}

# Run the test suit
[working-directory: './natrix']
test_setup t f:
    rustup run {{t}} cargo nextest run {{f}}
    rustup run {{t}} wasm-pack test --headless --chrome . {{f}}

# Format and run clippy against both stable and nightly
lint:
    cargo fmt --all
    cargo +stable clippy
    cargo +stable clippy --no-default-features
    cargo +nightly clippy --all-features

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
