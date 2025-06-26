//! Run the CI pipeline using dagger

use dagger_sdk::{Container, HostDirectoryOpts, Query};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dagger_sdk::connect(async |client| {
        tokio::try_join!(
            // doesnt_use_target(&client),
            // uses_stable_cache(&client),
            uses_nightly_cache(&client)
        )?;

        Ok(())
    })
    .await
    .map_err(Into::into)
}

/// uses the target caches(nightly)
async fn uses_nightly_cache(client: &Query) -> eyre::Result<()> {
    // run_native_tests(client).await?;
    run_web_unit_tests(client, Toolchain::Nightly).await?;

    // clippy_workspace(client).await?;
    // hack_clippy(client, Toolchain::Nightly).await?;

    Ok(())
}

/// uses the target caches(stable)
async fn uses_stable_cache(client: &Query) -> eyre::Result<()> {
    run_web_unit_tests(client, Toolchain::Stable).await?;
    hack_clippy(client, Toolchain::Stable).await?;

    Ok(())
}

/// Run the tasks that dont require target access
async fn doesnt_use_target(client: &Query) -> eyre::Result<()> {
    tokio::try_join!(
        check_for_outdated(client),
        check_for_typos(client),
        cargo_deny(client, "crates/natrix"),
        cargo_deny(client, "crates/natrix-cli"),
    )?;

    Ok(())
}

/// Run `cargo-deny` in the given folder
async fn cargo_deny(client: &Query, folder: &str) -> eyre::Result<()> {
    let container = Toolchain::Nightly.container(client);
    let container = install_binstall_tool(client, &container, "cargo-deny", None).await?;
    let container = with_workspace(client, &container);

    container
        .with_workdir(format!("/app/{folder}"))
        .with_exec(vec![
            "cargo",
            "deny",
            "--all-features",
            "check",
            "--exclude-dev",
        ])
        .exit_code()
        .await?;

    Ok(())
}

/// Check for outdated dependencies
async fn check_for_outdated(client: &Query) -> eyre::Result<()> {
    let container = Toolchain::Nightly.container(client);
    let container = install_binstall_tool(client, &container, "cargo-outdated", None).await?;
    let container = with_workspace(client, &container);

    container
        .with_exec(vec![
            "cargo-outdated",
            "outdated",
            "--root-deps-only",
            "--workspace",
            "--exit-code",
            "1",
        ])
        .exit_code()
        .await?;

    Ok(())
}

/// Run clippy on `/crates/natrix` for each features
async fn hack_clippy(client: &Query, toolchain: Toolchain) -> eyre::Result<()> {
    let container =
        toolchain
            .container(client)
            .with_exec(vec!["rustup", "component", "add", "clippy"]);
    let container = install_binstall_tool(client, &container, "cargo-hack", None).await?;
    let container = with_workspace(client, &container);
    let container = cache_target(client, &container, toolchain);

    let mut args = vec!["cargo", "hack", "clippy", "--each-feature"];
    if toolchain == Toolchain::Stable {
        args.push("--skip");
        args.push("nightly");
    }
    args.push("--");
    args.push("-Dwarnings");

    container
        .with_workdir("/app/crates/natrix")
        .with_exec(args)
        .exit_code()
        .await?;

    Ok(())
}

/// Run clippy on entire workspace
async fn clippy_workspace(client: &Query) -> eyre::Result<()> {
    let container = Toolchain::Nightly.container(client).with_exec(vec![
        "rustup",
        "component",
        "add",
        "clippy",
    ]);

    cache_target(
        client,
        &with_workspace(client, &container),
        Toolchain::Nightly,
    )
    .with_exec(vec![
        "cargo",
        "clippy",
        "--all-features",
        "--",
        "-Dwarnings",
    ])
    .exit_code()
    .await?;

    Ok(())
}

/// Run web unit tests
async fn run_web_unit_tests(client: &Query, toolchain: Toolchain) -> eyre::Result<()> {
    let container = toolchain.container(client).with_exec(vec![
        "rustup",
        "target",
        "add",
        "wasm32-unknown-unknown",
    ]);
    let container = setup_chrome(client, &container).await?;
    let container = install_binstall_tool(
        client,
        &container,
        "wasm-bindgen-cli",
        Some(vec!["wasm-bindgen", "wasm-bindgen-test-runner"]),
    )
    .await?;
    let container = install_binstall_tool(client, &container, "wasm-pack", None).await?;
    let container = with_workspace(client, &container);
    let container = cache_target(client, &container, toolchain);

    let mut args = vec![
        "tini",
        "--",
        "wasm-pack",
        "test",
        "--headless",
        "--chrome",
        "--mode",
        "no-install",
        "--features",
        "test_utils",
        "--features",
        "console_log",
    ];
    if toolchain == Toolchain::Nightly {
        args.push("--all-features");
    }

    container
        .with_workdir("/app/crates/natrix")
        .with_new_file(
            "./webdriver.json",
            r#"
{
  "goog:chromeOptions": {
    "args": [
      "--no-sandbox",
      "--user-data-dir=/tmp/data-dir",
      "--disable-dev-shm-usage",
      "--use-fake-device-for-media-stream",
      "--use-fake-ui-for-media-stream"
    ]
  }
}
            "#,
        )
        .with_exec(vec!["apt-get", "install", "-yqq", "tini"])
        .with_exec(args)
        .exit_code()
        .await?;

    Ok(())
}

/// Run natrix tests
async fn run_native_tests(client: &Query) -> eyre::Result<()> {
    let container = Toolchain::Nightly.container(client);
    let container = install_binstall_tool(client, &container, "cargo-nextest", None).await?;
    let container = with_workspace(client, &container);

    cache_target(client, &container, Toolchain::Nightly)
        .with_workdir("/app/crates/natrix")
        .with_exec(vec!["cargo", "nextest", "run", "--all-features"])
        .exit_code()
        .await?;

    Ok(())
}

/// Check the workspace for typos
async fn check_for_typos(client: &Query) -> eyre::Result<()> {
    let container = force_colors(&client.container().from("busybox"));
    with_workspace(
        client,
        &install_binstall_tool(client, &container, "typos-cli", Some(vec!["typos"])).await?,
    )
    .with_exec(vec!["typos"])
    .exit_code()
    .await?;

    Ok(())
}

/// Install chromeium and a chromedriver into the container
async fn setup_chrome(client: &Query, container: &Container) -> eyre::Result<Container> {
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
        .file(format!("/tmp/{archive_driver}/chromedriver"))
        .id()
        .await?;

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

    let chrome_devs = chrome
        .file(format!("/tmp/{archive_chrome}/deb.deps"))
        .id()
        .await?;
    let chrome = chrome
        .file(format!("/tmp/{archive_chrome}/chrome"))
        .id()
        .await?;

    let container = container
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
        .with_file("/bin/chrome", chrome);

    Ok(container)
}

/// With the workspace
fn with_workspace(client: &Query, container: &Container) -> Container {
    let workspace = client.host().directory_opts(
        ".",
        HostDirectoryOpts {
            exclude: Some(always_ignore()),
            include: None,
            no_cache: None,
        },
    );
    container
        .with_directory("/app", workspace)
        .with_workdir("/app")
}

/// Install the given tool to the container
async fn install_binstall_tool(
    client: &Query,
    container: &Container,
    tool: &str,
    outputs: Option<Vec<&str>>,
) -> eyre::Result<Container> {
    let outputs = outputs.unwrap_or_else(|| vec![tool]);

    let tool_container = Toolchain::Stable
        .container(client)
        .with_exec(vec!["apt-get", "install", "-y", "curl"])
        .with_exec(vec![
            "sh",
            "-c",
            // NOTE: We break this up because rustfmt chockes on long strings
            concat!(
                "curl -L --proto '=https' --tlsv1.2 -sSf ",
                "https://raw.githubusercontent.com/cargo-bins",
                "/cargo-binstall/main/install-from-binstall-release.sh",
                "| bash"
            ),
        ])
        .with_exec(vec![
            "cargo",
            "binstall",
            "-y",
            tool,
            "--install-path",
            "/bin",
        ]);

    let mut files = Vec::with_capacity(outputs.len());
    for output in outputs {
        let file = tool_container.file(format!("/bin/{output}")).id().await?;
        files.push(file);
    }

    Ok(container.with_files("/bin", files))
}

/// Return files to always ignore
fn always_ignore() -> Vec<&'static str> {
    vec!["target", "docs/book", "dist"]
}

/// The rust toolchain to use
#[derive(Clone, Copy, PartialEq, Eq)]
enum Toolchain {
    /// Stable toolchain
    Stable,
    /// Nightly toolchain
    Nightly,
}

impl Toolchain {
    /// Convert to str
    fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Nightly => "nightly",
        }
    }

    /// A container setup for rust
    fn container(self, client: &Query) -> Container {
        let image = match self {
            Self::Stable => "rust:1-slim",
            Self::Nightly => "rustlang/rust:nightly-slim",
        };

        cache_cargo_global(client, &force_colors(&client.container().from(image)))
            .with_exec(vec!["apt", "update"])
            .with_exec(vec!["rustup", "default", self.as_str()])
    }
}

/// Force all cli tools to display color
fn force_colors(container: &Container) -> Container {
    container
        .with_env_variable("FORCE_COLOR", "1")
        .with_env_variable("CLICOLOR_FORCE", "1")
        .with_env_variable("CARGO_TERM_COLOR", "always")
}

/// Cache rust
fn cache_target(client: &Query, container: &Container, toolchain: Toolchain) -> Container {
    container.with_mounted_cache(
        "./target",
        client.cache_volume(format!("rust-target-{}", toolchain.as_str())),
    )
}

/// cache the global cargo registers
fn cache_cargo_global(client: &Query, container: &Container) -> Container {
    container
        .with_mounted_cache(
            "/usr/local/cargo/registry",
            client.cache_volume("cargo-registry"),
        )
        .with_mounted_cache("/usr/local/cargo/git", client.cache_volume("cargo-git"))
}
