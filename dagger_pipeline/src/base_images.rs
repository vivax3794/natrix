//! Contains the base images

use dagger_sdk::{ContainerWithEnvVariableOpts, File};

use crate::prelude::*;

/// A base image
pub fn base(client: &Query) -> Container {
    client
        .container()
        .from("debian:bookworm-slim")
        .with_exec(vec!["apt", "update"])
        .with_mounted_cache("/file_caches", client.cache_volume("file_caches"))
}

/// A base image with rust installed
pub fn rust(client: &Query) -> Container {
    base(client)
        .with_exec(vec![
            "apt",
            "install",
            "-yqq",
            "curl",
            "build-essential",
            "libc6-dev",
        ])
        .with_exec(vec![
            "sh",
            "-c",
            "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y",
        ])
        .with_env_variable_opts(
            "PATH",
            "$PATH:/root/.cargo/bin",
            ContainerWithEnvVariableOpts { expand: Some(true) },
        )
        .with_mounted_cache(
            "/root/.cargo/registry",
            client.cache_volume("cargo-registry"),
        )
        .with_mounted_cache("/root/.cargo/git", client.cache_volume("cargo-git"))
        .with_symlink("/file_caches/package-cache", "/root/.cargo/.package-cache")
        .with_exec(vec!["rustup", "toolchain", "install", "nightly"])
}

/// Container for mdbook (already contains workspace)
pub fn book(client: &Query) -> Container {
    rust(client)
        .with_exec(vec!["rustup", "default", "nightly"])
        .with_exec(vec!["rustup", "component", "add", "rust-analyzer"])
        .with_exec(vec![
            "cargo",
            "install",
            "mdbookkit",
            "--features",
            "mdbook-rustdoc-link",
        ])
        .with_directory("/bin", binstall(client, "mdbook"))
        .with_directory("/bin", binstall(client, "mdbook-callouts"))
        .with_workspace(client)
        .with_workdir("./docs")
}

/// Base for wasm-tests
pub fn wasm(client: &Query) -> Container {
    chrome(client, &rust(client))
        .with_exec(vec![
            "rustup",
            "+nightly",
            "target",
            "add",
            "wasm32-unknown-unknown",
        ])
        .with_exec(vec![
            "rustup",
            "+stable",
            "target",
            "add",
            "wasm32-unknown-unknown",
        ])
        .with_exec(vec!["rustup", "+nightly", "component", "add", "rust-src"])
        .with_directory("/bin", binstall(client, "wasm-pack"))
        .with_directory("/bin", binstall(client, "wasm-bindgen-cli"))
        .with_file("/bin/wasm-opt", wasm_opt(client))
}

/// Instal wasm-opt
pub fn wasm_opt(client: &Query) -> File {
    // https://github.com/WebAssembly/binaryen/releases/download/version_123/binaryen-version_123-aarch64-linux.tar.gz
    let version = "123";
    let archive = format!("binaryen-version_{version}-x86_64-linux.tar.gz");
    let download_url = format!(
        "https://github.com/WebAssembly/binaryen/releases/download/version_{version}/{archive}"
    );

    base(client)
        .with_exec(vec!["apt", "install", "-y", "curl", "tar"])
        .with_exec(vec![
            "curl".into(),
            "-sSL".into(),
            download_url,
            "-o".into(),
            format!("/{archive}"),
        ])
        .with_exec(vec!["tar".into(), "-xzf".into(), format!("/{archive}")])
        .file(format!("/binaryen-version_{version}/bin/wasm-opt"))
}

/// Install chrome and chromedriver
fn chrome(client: &Query, container: &Container) -> Container {
    let chrome_version = "137.0.7151.119";
    let archive_driver = "chromedriver-linux64";
    let archive_chrome = "chrome-linux64";

    let download_driver = format!(
        "https://storage.googleapis.com/chrome-for-testing-public/{chrome_version}/linux64/{archive_driver}.zip"
    );
    let download_chrome = format!(
        "https://storage.googleapis.com/chrome-for-testing-public/{chrome_version}/linux64/{archive_chrome}.zip"
    );

    let base = base(client).with_exec(vec!["apt", "install", "-y", "curl", "unzip"]);
    let chromedriver = base
        .with_exec(vec![
            "curl".into(),
            "-sSl".into(),
            download_driver,
            "-o".into(),
            format!("/tmp/{archive_driver}.zip"),
        ])
        .with_exec(vec![
            "unzip".into(),
            format!("/tmp/{archive_driver}.zip"),
            "-d".into(),
            "/tmp/".into(),
        ])
        .file(format!("/tmp/{archive_driver}/chromedriver"));

    let chrome = base
        .with_exec(vec![
            "curl".into(),
            "-sSl".into(),
            download_chrome,
            "-o".into(),
            format!("/tmp/{archive_chrome}.zip"),
        ])
        .with_exec(vec![
            "unzip".into(),
            format!("/tmp/{archive_chrome}.zip"),
            "-d".into(),
            "/tmp/".into(),
        ]);

    let chrome_devs = chrome.file(format!("/tmp/{archive_chrome}/deb.deps"));
    let chrome_dir = chrome.directory(format!("/tmp/{archive_chrome}"));

    container
        .with_file("/tmp/chrome.deps", chrome_devs)
        .with_exec(vec![
            "sh",
            "-c",
            r#"
        while read pkg; do
          apt-get satisfy -y --no-install-recommends "${pkg}";
        done < /tmp/chrome.deps;"#,
        ])
        .with_file("/bin/chromedriver", chromedriver)
        .with_directory("/opt/chrome", chrome_dir)
        .with_exec(vec!["ln", "-s", "/opt/chrome/chrome", "/bin/chrome"])
}

/// Install a tool with bininstall and return a bin folder
pub fn binstall(client: &Query, tool: &'static str) -> Directory {
    rust(client)
        .with_exec(vec!["cargo", "install", "cargo-binstall"])
        .with_exec(vec![
            "cargo",
            "binstall",
            "-y",
            tool,
            "--install-path",
            "/result",
        ])
        .directory("/result")
}
