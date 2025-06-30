//! Actual targets

use std::collections::HashMap;

use dagger_sdk::{ContainerWithExecOptsBuilder, ReturnType};
use serde::Deserialize;

use crate::prelude::*;

// TODO: Clippy workspace
// TODO: Clippy hack natrix
// TODO: cargo-deny
// TODO: cargo-udeps
// TODO: cargo-outdated
// TODO: project gen test
// TODO: integration_tests

/// Lib test output
#[derive(Deserialize)]
struct TestLine {
    /// Ok/Failed
    event: String,
    /// Name
    name: String,
    /// The stdout
    stdout: Option<String>,
}

/// Run the native nextest tests
pub async fn native_tests(client: &Query, state: &GlobalState) -> Result<Directory> {
    let result = crate::base_images::rust(client)
        .with_directory(
            "/bin",
            crate::base_images::binstall(client, state, "cargo-nextest").await?,
        )
        .with_workspace(client)
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
        .with_env_variable("NEXTEST_EXPERIMENTAL_LIBTEST_JSON", "1")
        .with_exec_opts(
            vec![
                "cargo",
                "+nightly",
                "nextest",
                "run",
                "--all-features",
                "--offline",
                "--no-fail-fast",
                "--message-format",
                "libtest-json-plus",
                "--message-format-version",
                "0.1",
            ],
            ContainerWithExecOptsBuilder::default()
                .expect(ReturnType::Any)
                .build()?,
        )
        .stdout()
        .await?;

    let mut dir = client.directory();
    for line in result.lines() {
        if let Ok(json) = serde_json::from_str::<TestLine>(line) {
            if json.event == "started" {
                continue;
            }

            let status = if json.event == "ok" {
                None
            } else {
                Some(serde_json::json!({
                    "message": "Click for output",
                    "trace": json.stdout,
                }))
            };
            let id = uuid::Uuid::new_v4().to_string();
            let test = serde_json::json!({
                "uuid": id,
                "name": json.name,
                "status": if json.event == "ok" {"passed"} else {"failed"},
                "statusDetails": status,
                "labels": [
                    {
                        "name": "parentSuite",
                        "value":  "Unit Tests",
                    },
                    {
                        "name": "suite",
                        "value":  "Native",
                    },
                ]
            });
            let test = serde_json::to_string(&test)?;
            dir = dir.with_new_file(format!("{id}-result.json"), test);
        }
    }

    Ok(dir)
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
    let mut traces = HashMap::new();

    let mut hit_first = false;
    let mut current_name = String::new();
    let mut current_output = String::new();

    for line in output.lines() {
        if line.starts_with("----") {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            if let [_, name, _, _] = parts[..] {
                if hit_first {
                    let mut new_name = name.to_string();
                    std::mem::swap(&mut new_name, &mut current_name);
                    traces.insert(new_name, std::mem::take(&mut current_output));
                } else {
                    current_name = name.to_string();
                    hit_first = true;
                }
            }
        }

        if line.starts_with("test result") {
            traces.insert(current_name, current_output);
            break;
        }

        if hit_first {
            current_output.push_str(line);
            current_output.push('\n');
        }
    }

    let mut dir = client.directory();
    for line in output.lines() {
        if line.starts_with("test ") {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            if let [_test, name, _dots, result] = &parts[..] {
                let id = uuid::Uuid::new_v4().to_string();

                let status = if *result == "ok" {
                    None
                } else {
                    Some(serde_json::json!({
                        "message": "Click for output",
                        "trace": traces.get(*name),
                    }))
                };

                let test = serde_json::json!({
                    "uuid": id,
                    "name": name,
                    "status": if *result == "ok" {"passed"} else {"failed"},
                    "statusDetails": status,
                    "labels": [
                        {
                            "name": "parentSuite",
                            "value":  "Unit Tests",
                        },
                        {
                            "name": "suite",
                            "value":  "Web",
                        },
                        {
                            "name": "subSuite",
                            "value": toolchain
                        }
                    ]
                });
                let test = serde_json::to_string(&test)?;
                dir = dir.with_new_file(format!("{id}-result.json"), test);
            }
        }
    }
    Ok(dir)
}
