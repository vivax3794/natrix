# Run full CI pipeline with optional job count
full jobs="1":
    dagger --quiet run cargo run -p dagger_pipeline -- tests --jobs {{jobs}}

# Run quick CI pipeline with optional job count  
quick jobs="1":
    dagger --quiet run cargo run -p dagger_pipeline -- tests --quick --jobs {{jobs}}

# Apply automatic fixes
fix:
    dagger --quiet run cargo run -p dagger_pipeline -- fix

# Open mdbook
book:
    dagger --quiet run cargo run -p dagger_pipeline -- book
