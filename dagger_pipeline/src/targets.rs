//! Actual targets

use dagger_sdk::{ContainerWithExecOptsBuilder, ReturnType};

use crate::prelude::*;

/// Run the native nextest tests
pub async fn native_tests(client: &Query, state: &GlobalState) -> Result<File> {
    let result = crate::base_images::rust(client)
        .with_directory(
            "/bin",
            crate::base_images::binstall(client, state, "cargo-nextest").await?,
        )
        .with_workspace(client)
        .with_new_file(
            ".config/nextest.toml",
            r#"
[profile.ci]
fail-fast = false

[profile.ci.junit]
path = "junit.xml"
report-name = "Unit Tests"
"#,
        )
        .with_workdir("./crates/natrix")
        .run_with_mutex(
            state,
            vec![
                "cargo",
                "+nightly",
                "nextest",
                "run",
                "--all-features",
                "--no-run",
            ],
            false,
        )
        .await?
        .with_exec_opts(
            vec![
                "cargo",
                "+nightly",
                "nextest",
                "run",
                "--all-features",
                "--offline",
                "--profile",
                "ci",
            ],
            ContainerWithExecOptsBuilder::default()
                .expect(ReturnType::Any)
                .build()?,
        )
        // NOTE: Dagger doesnt seem to like using `.file` on stuff in a cache
        .with_exec(vec!["mv", "/app/target/nextest/ci/junit.xml", "/junit.xml"])
        .with_exec(vec![
            "sed",
            "-i",
            r#"s/"natrix"/"Unit Tests"/g"#,
            "/junit.xml",
        ])
        .file("/junit.xml");

    Ok(result)
}

/// Wasm unit tests
pub async fn wasm_unit_tests(
    client: &Query,
    state: &GlobalState,
    toolchain: &'static str,
) -> Result<Directory> {
    let output = crate::base_images::wasm(client, state)
        .await?
        .with_workspace(client)
        .with_workdir("./crates/natrix")
        .run_with_mutex(
            state,
            vec![
                "cargo",
                if toolchain == "nightly" {
                    "+nightly"
                } else {
                    "+stable"
                },
                "build",
                "--tests",
                "--target",
                "wasm32-unknown-unknown",
                "--features",
                "test_utils",
                if toolchain == "nightly" {
                    "--all-features"
                } else {
                    "--" // Just a valid argument
                },
            ],
            false,
        )
        .await?
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
      "--use-fake-ui-for-media-stream",
      "--headless"
    ]
  }
}
            "#,
        )
        .with_exec_opts(
            vec![
                "rustup",
                "run",
                toolchain,
                "wasm-pack",
                "test",
                "--headless",
                "--chrome",
                "--features",
                "test_utils",
                if toolchain == "nightly" {
                    "--all-features"
                } else {
                    "--" // Just a valid argument
                },
            ],
            ContainerWithExecOptsBuilder::default()
                .expect(ReturnType::Any)
                .build()?,
        )
        .stdout()
        .await?;

    let dir = extract_test_result(client, toolchain, &output)?;

    Ok(dir)
}

/// Extract the result of a `wasm-pack test` run
fn extract_test_result(
    client: &Query,
    toolchain: &'static str,
    output: &str,
) -> Result<Directory, eyre::Error> {
    let mut dir = client.directory();
    let mut results = Vec::new();
    for line in output.lines() {
        if line.starts_with("test ") {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            if let [_test, name, _dots, result] = &parts[..] {
                let id = uuid::Uuid::new_v4().to_string();

                let test = serde_json::json!({
                    "uuid": id,
                    "name": name,
                    "status": if *result == "ok" {"passed"} else {"failed"},
                    "statusDetails": {
                        "message": "See stack trace",
                        // MAYBE: Capture the induvidual test case failrues
                        "trace": if *result == "ok" {"ok"} else {&output},
                    },
                    "labels": [
                        {
                            "name": "suite",
                            "value":  "Web Unit Tests",
                        },
                        {
                            "name": "subSuite",
                            "value": toolchain
                        }
                    ]
                });
                let test = serde_json::to_string(&test)?;
                dir = dir.with_new_file(format!("{id}-result.json"), test);
                results.push(id);
            }
        }
    }
    Ok(dir)
}
