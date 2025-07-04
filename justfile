# Run full CI pipeline with optional job count
full jobs="1":
    dagger --quiet run cargo run -p dagger_pipeline -- --jobs {{jobs}}

# Run quick CI pipeline with optional job count  
quick jobs="1":
    dagger --quiet run cargo run -p dagger_pipeline -- --quick --jobs {{jobs}}