update_snapshot:
    cargo insta test --review --package natrix --unreferenced delete --test-runner nextest --all-features

fix_typos:
    typos -w

docs:
    cargo doc --open -p natrix --lib --all-features

book:
    earthly --secret GITHUB_TOKEN="" ./docs+build-book
    cd ./docs/book && python3 -m http.server

core:
    earthly --secret GITHUB_TOKEN="" +run-core

all:
    earthly --secret GITHUB_TOKEN="" +all

install_cli:
    cargo install --path ./crates/natrix-cli

[working-directory: './ci/stress_test_binary_size']
stress_size: install_cli
    @echo
    @echo "--- Checking initial size (if file exists)..."
    @wc -c dist/code_bg.wasm || echo "No initial file."

    @echo
    @echo "--- Building Wasm with 'natrix build'..."
    @natrix build

    @echo
    @echo "--- Compression Size Report ---"
    @( \
        set -e; \
        UNCOMPRESSED=$(wc -c < dist/code_bg.wasm | tr -d ' '); \
        JS_FILE=$(wc -c < dist/code.js | tr -d ' '); \
        GZIPPED=$(gzip --stdout --best dist/code_bg.wasm | wc -c | tr -d ' '); \
        BROTLI=$(brotli --stdout --best dist/code_bg.wasm | wc -c | tr -d ' '); \
        \
        printf "Uncompressed : %'d bytes\n" $UNCOMPRESSED; \
        printf "Gzip (-9)    : %'d bytes\n" $GZIPPED; \
        printf "Brotli (-11) : %'d bytes\n" $BROTLI; \
        printf "\n"; \
        printf "JS           : %'d bytes\n" $JS_FILE; \
    )


[working-directory: './ci/benchmark']
bench: install_cli
    #!/usr/bin/bash
    set -e
    cleanup() {
        echo "Cleaning up..."
        if [ -n "$python_pid" ]; then
            kill "$python_pid" 2>/dev/null || true
        fi
    }
    trap cleanup EXIT

    natrix build
    (cd dist && python3 -m http.server 8888 2>/dev/null) &
    python_pid=$!

    wasm_bench
