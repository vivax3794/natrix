# Run full CI pipeline with optional job count
full jobs="1":
    dagger --quiet run cargo run -p dagger_pipeline -- tests --jobs {{jobs}}

# Run quick CI pipeline with optional job count  
quick jobs="1":
    dagger --quiet run cargo run -p dagger_pipeline -- tests --filter rustfmt,typos,native_tests,wasm_unit_nightly,clippy_workspace --jobs {{jobs}}

# Documentation-related tests
docs jobs="1":
    dagger --quiet run cargo run -p dagger_pipeline -- tests --filter typos,test_docs,test_book_links,test_book_examples --jobs {{jobs}}

# CLI and integration tests
cli jobs="1":
    dagger --quiet run cargo run -p dagger_pipeline -- tests --filter test_project_gen_stable,test_project_gen_nightly,integration_test_dev,integration_test_release,integration_test_build --jobs {{jobs}}

# All linting and static analysis
lint jobs="1":
    dagger --quiet run cargo run -p dagger_pipeline -- tests --filter rustfmt,typos,clippy_workspace,clippy_natrix_nightly,clippy_natrix_stable,cargo_deny_natrix,cargo_deny_natrix_cli,unused_deps,outdated_deps --jobs {{jobs}}

# Apply automatic fixes
fix:
    dagger --quiet run cargo run -p dagger_pipeline -- fix

# Open mdbook
book:
    dagger --quiet run cargo run -p dagger_pipeline -- book

# Run benchmarks 
bench:
    dagger --quiet run cargo run -p dagger_pipeline -- bench
