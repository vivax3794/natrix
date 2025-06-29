//! Contains the base images

use dagger_sdk::ContainerWithEnvVariableOpts;

use crate::prelude::*;

/// A base image
pub fn base(client: &Query) -> Container {
    client
        .container()
        .from("debian:bookworm-slim")
        .with_env_variable("FORCE_COLOR", "1")
        .with_env_variable("CLICOLOR_FORCE", "1")
        .with_env_variable("CARGO_TERM_COLOR", "always")
        .with_exec(vec!["apt", "update"])
}

/// A base image with rust installed
pub fn rust(client: &Query) -> Container {
    base(client)
        .with_exec(vec!["apt", "install", "-yqq", "curl", "build-essential"])
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
        .with_exec(vec!["rustup", "toolchain", "install", "nightly"])
}

/// Base for wasm-tests
pub async fn wasm(client: &Query, state: &GlobalState) -> Result<Container> {
    let (wasm_pack, wasm_bindgen) = tokio::try_join!(
        binstall(client, state, "wasm-pack"),
        binstall(client, state, "wasm-bindgen-cli")
    )?;

    let container = chrome(client, &rust(client))
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
        .with_directory("/bin", wasm_pack)
        .with_directory("/bin", wasm_bindgen);

    Ok(container)
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

    let base = client
        .container()
        .from("debian:bookworm-slim")
        .with_exec(vec!["apt", "update"])
        .with_exec(vec!["apt", "install", "-y", "curl", "unzip"]);
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
pub async fn binstall(
    client: &Query,
    state: &GlobalState,
    tool: &'static str,
) -> Result<Directory> {
    let result = rust(client)
        .run_with_mutex(state, vec!["cargo", "install", "cargo-binstall"], false)
        .await?
        .with_exec(vec![
            "cargo",
            "binstall",
            "-y",
            tool,
            "--install-path",
            "/result",
        ])
        .directory("/result");
    Ok(result)
}
