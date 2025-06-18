VERSION 0.8
IMPORT github.com/earthly/lib/rust:3.0.3 AS rust

# ========== REUSABLE FUNCTION ============

rust:
    ARG --required toolchain
    FROM busybox
    IF [ "$toolchain" = "nightly" ]
        FROM rustlang/rust:nightly-slim
    ELSE
        FROM rust:1-slim
    END

    ENV RUSTUP_TOOLCHAIN=$toolchain
    ENV CARGO_TERM_COLOR=always
    WORKDIR /ci/

    # https://github.com/earthly/lib/issues/65
    RUN cargo install --locked --git https://github.com/holmgr/cargo-sweep cargo-sweep
    DO rust+INIT --keep_fingerprints=true

with-clippy:
    ARG --required toolchain
    FROM +rust --toolchain=$toolchain
    RUN rustup component add clippy

wasm:
    ARG --required toolchain
    FROM +rust --toolchain=$toolchain

    RUN rustup target add wasm32-unknown-unknown
    IF  [ "$toolchain" = "nightly" ]
        RUN rustup component add rust-src
    END

    RUN apt-get update -qq
    RUN apt-get install --no-install-recommends -qq chromium

    COPY (+install-tool/tool --tool=wasm-bindgen-cli --name=wasm-bindgen) /bin/wasm-bindgen
    COPY (+install-tool/tool --tool=wasm-bindgen-cli --name=wasm-bindgen-test-runner) /bin/wasm-bindgen-test-runner
    COPY (+install-wasm-opt/wasm-opt) /bin/wasm-opt

install-wasm-opt:
    FROM debian:bookworm-slim
    ENV BINARYEN_VERSION="123"
    ENV ARCHIVE_NAME="binaryen-version_${BINARYEN_VERSION}-x86_64-linux.tar.gz"
    ENV DOWNLOAD_URL="https://github.com/WebAssembly/binaryen/releases/download/version_${BINARYEN_VERSION}/${ARCHIVE_NAME}"

    RUN apt-get update -qq
    RUN apt-get install --no-install-recommends -qq ca-certificates curl

    RUN curl -sSL ${DOWNLOAD_URL} -o /tmp/${ARCHIVE_NAME} \
        && tar -xzf /tmp/${ARCHIVE_NAME} -C /tmp/ \
        && rm /tmp/${ARCHIVE_NAME}

    SAVE ARTIFACT /tmp/binaryen-version_${BINARYEN_VERSION}/bin/wasm-opt wasm-opt

install-tool-base:
    FROM +rust --toolchain=stable

    RUN apt-get update -qq
    RUN apt-get install --no-install-recommends -qq curl
    RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

    RUN mkdir ./result

install-tool:
    ARG --required tool
    ARG name=$tool
    FROM +install-tool-base
    RUN cargo binstall $tool --install-path ./result
    SAVE ARTIFACT result/$name tool

workspace-src:
    FROM scratch
    COPY --keep-ts --dir . ./src/
    SAVE ARTIFACT --keep-ts ./src src

# =============== TARGETS ================

fmt:
    FROM +rust --toolchain=nightly
    RUN rustup component add rustfmt
    COPY +workspace-src/src .
    RUN cargo fmt --check

typos:
    FROM busybox
    WORKDIR /ui/

    COPY (+install-tool/tool --tool=typos-cli --name=typos) /bin/typos
    COPY +workspace-src/src .
    RUN typos

unused-dependencies:
    FROM +rust --toolchain=nightly
    COPY (+install-tool/tool --tool=cargo-hack) /bin/cargo-hack
    COPY (+install-tool/tool --tool=cargo-udeps) /bin/cargo-udeps
    COPY --keep-ts +workspace-src/src .

    DO rust+CARGO --args="hack udeps --each-feature --ignore-private --all-targets"

outdated-dependencies:
    FROM +rust --toolchain=nightly
    COPY (+install-tool/tool --tool=cargo-outdated) /bin/cargo-outdated
    COPY --keep-ts +workspace-src/src .

    DO rust+CARGO --args="outdated -R --workspace"

check-docs:
    FROM +rust --toolchain=nightly
    COPY --keep-ts +workspace-src/src .
    DO rust+CARGO --args="test --doc --all-features --workspace "

# ========== Entry Points ===========

run-core:
    BUILD ./crates/natrix+test-native-dev
    BUILD ./crates/natrix+test-web --toolchain=nightly

all:
    WAIT
        BUILD ./crates+all
        BUILD +fmt
        BUILD +typos
        BUILD +unused-dependencies
        BUILD +outdated-dependencies
        BUILD +check-docs
        BUILD ./ci/integration_tests+test-production
    END

    WAIT
        BUILD ./ci/integration_tests+test-dev
    END
    WAIT
        BUILD ./ci/integration_tests+test-release
    END
    WAIT
        BUILD ./docs+test-examples
    END
