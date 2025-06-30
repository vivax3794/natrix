//! Actual targets

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::prelude::*;

// TODO: integration_tests
// MAYBE: Unused features
// MAYBE: Record timing info
// MAYBE: Add execution info
// TODO: Run `typos -w` and apply fixes
// TODO: Update snapshots.
// TODO: Build book
// TODO: Build docs

/// Test suite names
const UNIT_TESTS: &str = "Unit Tests";
/// Linters test suite name
const LINTERS: &str = "Linters";
/// End to end tests suite name
const END_TO_END_TESTS: &str = "End To End Tests";

/// Test status
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum TestStatus {
    /// Test passed
    Passed,
    /// Test failed
    Failed,
    /// Test skipped
    Skipped,
}

/// Test event result types
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum TestEventResult {
    /// Test passed
    Ok,
    /// Test failed
    Failed,
    /// Test ignored
    Ignored,
}

/// Test event types
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(untagged)]
enum TestEvent {
    /// Test started (ignored in processing)
    Started,
    /// Test completed with result
    Result(TestEventResult),
}

/// Label types
#[derive(Serialize)]
#[serde(tag = "name", content = "value")]
enum LabelType {
    /// Suite label
    #[serde(rename = "suite")]
    Suite(String),
    /// Sub-suite label
    #[serde(rename = "subSuite")]
    SubSuite(String),
}

/// Lib test output
#[derive(Deserialize)]
struct TestLine {
    /// Test event type
    event: TestEvent,
    /// Name
    name: String,
    /// The stdout
    stdout: Option<String>,
}

/// Status details for failed tests
#[derive(Serialize)]
struct StatusDetails {
    /// Error message
    message: String,
    /// Optional trace information
    #[serde(skip_serializing_if = "Option::is_none")]
    trace: Option<String>,
}

/// Test step
#[derive(Serialize)]
struct Step {
    /// Step name
    name: String,
    /// Step status
    status: TestStatus,
    /// Optional status details for failed steps
    #[serde(skip_serializing_if = "Option::is_none", rename = "statusDetails")]
    status_details: Option<StatusDetails>,
}

/// Test result for allure reporting
#[derive(Serialize)]
struct TestResult {
    /// Unique test identifier
    uuid: String,
    /// Test name
    name: String,
    /// Test status (passed/failed)
    status: TestStatus,
    /// Optional status details for failed tests
    #[serde(skip_serializing_if = "Option::is_none", rename = "statusDetails")]
    status_details: Option<StatusDetails>,
    /// Test labels
    labels: Vec<LabelType>,
    /// Optional test steps
    #[serde(skip_serializing_if = "Option::is_none")]
    steps: Option<Vec<Step>>,
}

impl TestResult {
    /// Convert test result to a directory containing the result file
    fn into_file(self, client: &Query) -> Result<Directory> {
        let content = serde_json::to_string(&self)?;
        let filename = format!("{}-result.json", self.uuid);
        Ok(client.directory().with_new_file(filename, content))
    }
}

/// Parse libtest JSON output and create test result directory
fn parse_libtest_json(client: &Query, output: &str, suite_name: &str) -> Result<Directory> {
    let mut dir = client.directory();
    for line in output.lines() {
        if let Ok(json) = serde_json::from_str::<TestLine>(line) {
            let event_result = match json.event {
                TestEvent::Started => continue,
                TestEvent::Result(result) => result,
            };

            let status_details = match event_result {
                TestEventResult::Ok => None,
                TestEventResult::Failed | TestEventResult::Ignored => Some(StatusDetails {
                    message: json.stdout.unwrap_or_default(),
                    trace: None,
                }),
            };

            let test_result = TestResult {
                uuid: uuid::Uuid::new_v4().to_string(),
                name: json
                    .name
                    .strip_prefix("natrix::natrix$")
                    .unwrap_or(&json.name)
                    .to_string(),
                status: match event_result {
                    TestEventResult::Ok => TestStatus::Passed,
                    TestEventResult::Failed => TestStatus::Failed,
                    TestEventResult::Ignored => TestStatus::Skipped,
                },
                status_details,
                labels: vec![
                    LabelType::Suite(UNIT_TESTS.to_string()),
                    LabelType::SubSuite(suite_name.to_string()),
                ],
                steps: None,
            };

            dir = dir.with_directory(".", test_result.into_file(client)?);
        }
    }
    Ok(dir)
}

/// Generic linter configuration
struct LinterConfig {
    /// Name of the linter test
    name: String,
    /// Command to run
    command: Vec<&'static str>,
    /// Working directory to run in
    workdir: Option<&'static str>,
    /// Whether to use rust image or busybox
    use_rust_image: bool,
    /// Tools to install via binstall
    needs_binstall: Vec<&'static str>,
    /// Whether to capture stderr or stdout
    capture_stderr: bool,
}

impl Default for LinterConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            command: Vec::new(),
            workdir: None,
            use_rust_image: true,
            needs_binstall: Vec::new(),
            capture_stderr: true,
        }
    }
}

/// Run a generic linter and return test result directory
async fn run_linter(client: &Query, config: LinterConfig) -> Result<Directory> {
    let mut container = if config.use_rust_image {
        crate::base_images::rust(client)
    } else {
        client.container().from("busybox")
    };

    for tool in &config.needs_binstall {
        container = container.with_directory("/bin", crate::base_images::binstall(client, tool));
    }

    container = container.with_workspace(client);

    if let Some(workdir) = config.workdir {
        container = container.with_workdir(workdir);
    }

    let result = container.with_exec_any(config.command)?;

    let exec_result = result.get_result().await?;
    let status = if exec_result.exit_code == 0 {
        None
    } else {
        let (output, other) = if config.capture_stderr {
            (exec_result.stderr, exec_result.stdout)
        } else {
            (exec_result.stdout, exec_result.stderr)
        };
        Some(StatusDetails {
            message: output,
            trace: Some(other),
        })
    };

    let test_result = TestResult {
        uuid: uuid::Uuid::new_v4().to_string(),
        name: config.name,
        status: if exec_result.exit_code == 0 {
            TestStatus::Passed
        } else {
            TestStatus::Failed
        },
        status_details: status,
        labels: vec![LabelType::Suite(LINTERS.to_string())],
        steps: None,
    };

    test_result.into_file(client)
}

/// Run the native nextest tests
pub async fn native_tests(client: &Query) -> Result<Directory> {
    let result = crate::base_images::rust(client)
        .with_directory(
            "/bin",
            crate::base_images::binstall(client, "cargo-nextest"),
        )
        .with_workspace(client)
        .with_workdir("./crates/natrix")
        .with_exec(vec![
            "cargo",
            "+nightly",
            "nextest",
            "run",
            "--all-features",
            "--no-run",
        ])
        .with_env_variable("NEXTEST_EXPERIMENTAL_LIBTEST_JSON", "1")
        .with_exec_any(vec![
            "cargo",
            "+nightly",
            "nextest",
            "run",
            "--color",
            "never",
            "--all-features",
            "--offline",
            "--no-fail-fast",
            "--message-format",
            "libtest-json-plus",
            "--message-format-version",
            "0.1",
        ])?
        .stdout()
        .await?;

    let dir = parse_libtest_json(client, &result, "Native")?;
    Ok(dir)
}

/// Wasm unit tests
pub async fn wasm_unit_tests(client: &Query, toolchain: &'static str) -> Result<Directory> {
    let output = crate::base_images::wasm(client)
        .with_workspace(client)
        .with_workdir("./crates/natrix")
        .with_exec(vec![
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
            "_internal_testing",
            if toolchain == "nightly" {
                "--all-features"
            } else {
                "--" // Just a valid argument
            },
        ])
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
        .with_exec_any(vec![
            "rustup",
            "run",
            toolchain,
            "wasm-pack",
            "test",
            "--headless",
            "--chrome",
            "--features",
            "_internal_testing",
            if toolchain == "nightly" {
                "--all-features"
            } else {
                "--" // Just a valid argument
            },
        ])?
        .stdout()
        .await?;

    let dir = parse_wasm_pack_output(client, toolchain, &output)?;

    Ok(dir)
}

/// Extract the result of a `wasm-pack test` run
fn parse_wasm_pack_output(
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

                let status_details = if *result == "ok" {
                    None
                } else {
                    traces.get(*name).map(|output| StatusDetails {
                        message: output.clone(),
                        trace: None,
                    })
                };

                let test_result = TestResult {
                    uuid: id,
                    name: name
                        .strip_prefix("actual_tests::")
                        .unwrap_or(name)
                        .to_string(),
                    status: if *result == "ok" {
                        TestStatus::Passed
                    } else {
                        TestStatus::Failed
                    },
                    status_details,
                    labels: vec![
                        LabelType::Suite(UNIT_TESTS.to_string()),
                        LabelType::SubSuite(format!("Web {toolchain}")),
                    ],
                    steps: None,
                };

                dir = dir.with_directory(".", test_result.into_file(client)?);
            }
        }
    }
    Ok(dir)
}

/// Run rustfmt on the workspace
pub async fn rustfmt(client: &Query) -> Result<Directory> {
    run_linter(
        client,
        LinterConfig {
            name: "Rust-fmt".to_string(),
            command: vec!["cargo", "+nightly", "fmt", "--check"],
            capture_stderr: false,
            ..Default::default()
        },
    )
    .await
}

/// Run typos on the workspace
pub async fn typos(client: &Query) -> Result<Directory> {
    run_linter(
        client,
        LinterConfig {
            name: "Spell Checking".to_string(),
            command: vec!["typos", "--color", "never", "--format", "brief"],
            use_rust_image: false,
            needs_binstall: vec!["typos-cli"],
            capture_stderr: false,
            ..Default::default()
        },
    )
    .await
}

/// Run clippy on the workspace
pub async fn clippy_workspace(client: &Query) -> Result<Directory> {
    run_linter(
        client,
        LinterConfig {
            name: "Clippy All".to_string(),
            command: vec![
                "cargo",
                "+nightly",
                "clippy",
                "--all-features",
                "--tests",
                "--",
                "-Dwarnings",
            ],
            ..Default::default()
        },
    )
    .await
}

/// Run clippy hack on natrix
pub async fn clippy_natrix(client: &Query, toolchain: &'static str) -> Result<Directory> {
    let mut command = vec![
        "cargo",
        if toolchain == "nightly" {
            "+nightly"
        } else {
            "+stable"
        },
        "hack",
        "clippy",
        "--each-feature",
    ];
    if toolchain == "stable" {
        command.extend(["--skip", "nightly"]);
    }
    command.extend(["--tests", "--", "-Dwarnings"]);

    run_linter(
        client,
        LinterConfig {
            name: format!("Clippy hack Natrix ({toolchain})"),
            command,
            workdir: Some("./crates/natrix"),
            needs_binstall: vec!["cargo-hack"],
            ..Default::default()
        },
    )
    .await
}

/// Run `cargo-deny` on the given folder
pub async fn cargo_deny(client: &Query, folder: &'static str) -> Result<Directory> {
    run_linter(
        client,
        LinterConfig {
            name: format!("Cargo deny ({folder})"),
            command: vec!["cargo", "deny", "check", "--exclude-dev"],
            workdir: Some(folder),
            needs_binstall: vec!["cargo-deny"],
            ..Default::default()
        },
    )
    .await
}

/// Run `cargo-udeps` on the given folder
pub async fn unused_deps(client: &Query) -> Result<Directory> {
    run_linter(
        client,
        LinterConfig {
            name: "Unused dependencies".to_string(),
            needs_binstall: vec!["cargo-udeps", "cargo-hack"],
            command: vec![
                "cargo",
                "+nightly",
                "hack",
                "udeps",
                "--each-feature",
                "--all-targets",
            ],
            capture_stderr: false,
            ..Default::default()
        },
    )
    .await
}

/// Run `cargo-udeps` on the given folder
pub async fn outdated_deps(client: &Query) -> Result<Directory> {
    run_linter(
        client,
        LinterConfig {
            name: "Outdated dependencies".to_string(),
            needs_binstall: vec!["cargo-outdated"],
            command: vec![
                "cargo",
                "outdated",
                "--workspace",
                "--root-deps-only",
                "--exit-code",
                "1",
            ],
            capture_stderr: false,
            ..Default::default()
        },
    )
    .await
}

/// Build the cli
pub fn cli(client: &Query) -> Directory {
    let file = crate::base_images::rust(client)
        .with_workspace(client)
        .with_exec(vec![
            "cargo",
            "build",
            "--profile",
            "dev",
            "-p",
            "natrix-cli",
        ])
        .with_exec(vec!["mv", "./target/debug/natrix", "/tmp/natrix"])
        .file("/tmp/natrix");
    client.directory().with_file("natrix", file)
}

/// Test that the natrix project generator works
pub async fn test_project_gen(client: &Query, toolchain: &'static str) -> Result<Directory> {
    let container = crate::base_images::wasm(client)
        .with_exec(vec!["rustup", "default", toolchain])
        .with_directory("/bin", cli(client))
        .with_env_variable("NATRIX_PATH", "/app/crates/natrix")
        .with_exec_any(vec![
            "natrix",
            "new",
            "project",
            if toolchain == "stable" {
                "--stable"
            } else {
                "--"
            },
        ])?
        .with_workspace(client)
        .with_workdir("/project")
        .with_mounted_cache(
            "/project/target",
            client.cache_volume(format!("project-gen-test-target-{toolchain}")),
        );

    let mut success = true;
    let mut steps = Vec::with_capacity(3);

    let gen_result = container.get_result().await?;
    if gen_result.exit_code == 0 {
        steps.push(Step {
            name: "Generate project".to_string(),
            status: TestStatus::Passed,
            status_details: None,
        });

        let build = container.with_exec_any(vec!["natrix", "build"])?;
        let test = container.with_exec_any(vec!["wasm-pack", "test", "--headless", "--chrome"])?;

        let (build_result, test_result) = tokio::try_join!(build.get_result(), test.get_result())?;

        if build_result.exit_code == 0 {
            steps.push(Step {
                name: "Build project".to_string(),
                status: TestStatus::Passed,
                status_details: None,
            });
        } else {
            success = false;
            steps.push(Step {
                name: "Build project".to_string(),
                status: TestStatus::Failed,
                status_details: Some(StatusDetails {
                    message: build_result.stderr,
                    trace: Some(build_result.stdout),
                }),
            });
        }
        if test_result.exit_code == 0 {
            steps.push(Step {
                name: "Test project".to_string(),
                status: TestStatus::Passed,
                status_details: None,
            });
        } else {
            success = false;
            steps.push(Step {
                name: "Test project".to_string(),
                status: TestStatus::Failed,
                status_details: Some(StatusDetails {
                    message: test_result.stderr,
                    trace: Some(test_result.stdout),
                }),
            });
        }
    } else {
        success = false;
        steps.push(Step {
            name: "Generate project".to_string(),
            status: TestStatus::Failed,
            status_details: Some(StatusDetails {
                message: gen_result.stderr,
                trace: Some(gen_result.stdout),
            }),
        });
    }

    let test_result = TestResult {
        uuid: uuid::Uuid::new_v4().to_string(),
        name: format!("Project gen ({toolchain})"),
        status: if success {
            TestStatus::Passed
        } else {
            TestStatus::Failed
        },
        status_details: None,
        labels: vec![LabelType::Suite(END_TO_END_TESTS.to_string())],
        steps: Some(steps),
    };

    test_result.into_file(client)
}

/// Run the cargo doc tests
pub async fn test_docs(client: &Query) -> Result<Directory> {
    let output = crate::base_images::rust(client)
        .with_workspace(client)
        .with_exec_any(vec![
            "cargo",
            "+nightly",
            "test",
            "--all-features",
            "--doc",
            "--no-fail-fast",
            "--",
            "-Z",
            "unstable-options",
            "--format",
            "json",
        ])?
        .stdout()
        .await?;

    parse_libtest_json(client, &output, "Documentation")
}

/// Test the links in the mdbooks
pub async fn test_book_links(client: &Query) -> Result<Directory> {
    let result = crate::base_images::book(client)
        .with_exec_any(vec!["mdbook", "build"])?
        .get_result()
        .await?;

    let status = if result.exit_code == 0 {
        None
    } else {
        Some(StatusDetails {
            message: result.stderr,
            trace: None,
        })
    };

    let result = TestResult {
        uuid: uuid::Uuid::new_v4().to_string(),
        status: if result.exit_code == 0 {
            TestStatus::Passed
        } else {
            TestStatus::Failed
        },
        name: "Book Links".to_string(),
        status_details: status,
        labels: vec![LabelType::Suite(LINTERS.to_string())],
        steps: None,
    };
    result.into_file(client)
}

/// Run the example tests in the book
/// INVARIANT: This test is extremely flaky in terms of cache state.
/// So should not be run in parallel.
pub async fn test_book_examples(client: &Query) -> Result<Directory> {
    let result = crate::base_images::book(client)
        .with_exec_any(vec!["sh", "-c", "rm -r ../target/debug/deps/*natrix*"])?
        .with_exec_any(vec![
            "sh",
            "-c",
            "rm -r ../target/debug/deps/*wasm_bindgen_test*",
        ])?
        .with_exec(vec![
            "cargo",
            "build",
            "-p",
            "natrix",
            "--all-features",
            "--tests",
        ])
        .with_exec_any(vec!["mdbook", "test", "-L", "../target/debug/deps"])?
        .get_result()
        .await?;

    let status = if result.exit_code == 0 {
        None
    } else {
        Some(StatusDetails {
            message: result.stderr,
            trace: None,
        })
    };

    let result = TestResult {
        uuid: uuid::Uuid::new_v4().to_string(),
        status: if result.exit_code == 0 {
            TestStatus::Passed
        } else {
            TestStatus::Failed
        },
        name: "Book Examples".to_string(),
        status_details: status,
        labels: vec![LabelType::Suite(UNIT_TESTS.to_string())],
        steps: None,
    };
    result.into_file(client)
}
