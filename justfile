# Run full CI pipeline with optional job count
test jobs="1":
    dagger --quiet run cargo run -p dagger_pipeline -- tests --jobs {{jobs}}

# Run full CI pipeline with optional job count, just print out the result
test_tui jobs="1":
    dagger --quiet run cargo run -p dagger_pipeline -- tests --jobs {{jobs}} --tui

# Apply automatic fixes
fix:
    dagger --quiet run cargo run -p dagger_pipeline -- fix

# Open mdbook
book:
    dagger --quiet run cargo run -p dagger_pipeline -- book

# Run benchmarks 
bench:
    dagger --quiet run cargo run -p dagger_pipeline -- bench

