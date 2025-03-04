# Lint and run tests
check: test lint

# Run the test suit
[working-directory: './natrix']
test:
    rustup run stable cargo nextest run
    rustup run stable cargo nextest run --all-features
    rustup run stable wasm-pack test --headless --chrome 
    rustup run nightly wasm-pack test --headless --chrome --features nightly

# Format and run clippy against both stable and nightly
lint:
    cargo fmt --all
    cargo +stable clippy
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


# Publish the crate to crates.io
publish: check
    cargo publish -p natrix_macros
    cargo publish -p natrix

# Remove all build artifacts
clean:
    cargo clean
    rm -rv docs/book || true
