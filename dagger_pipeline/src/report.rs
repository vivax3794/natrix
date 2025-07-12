//! Construct CI report.

use std::pin::Pin;

use dagger_sdk::{ContainerWithExecOptsBuilder, PortForward, ReturnType, ServiceUpOpts};
use futures::{StreamExt, stream};

use crate::TestCommand;
use crate::prelude::*;
use crate::targets::IntegrationTestMode;

/// Run all tests and return a directory of allure reports
pub async fn run_all_tests(client: &Query, arguments: &TestCommand) -> Result<Directory> {
    // Create all possible test futures with their names
    let all_tests = all_tests(client, arguments);

    let mut tasks: Vec<Pin<Box<dyn Future<Output = _>>>> = vec![];
    let mut need_sequential: Vec<Pin<Box<dyn Future<Output = _>>>> = vec![];

    // Filter tests based on arguments
    for (name, future, is_sequential) in all_tests {
        let should_run = if arguments.filter.is_empty() {
            true // Run all tests if no filter
        } else {
            arguments.filter.iter().any(|filter| filter == name)
        };

        if should_run {
            if is_sequential {
                need_sequential.push(future);
            } else {
                tasks.push(future);
            }
        }
    }

    let mut dir = client.directory();
    let mut stream = stream::iter(tasks).buffer_unordered(arguments.jobs.unwrap_or(1));
    while let Some(task) = stream.next().await {
        dir = dir.with_directory(".", task?.sync().await?);
    }

    for task in need_sequential {
        dir = dir.with_directory(".", task.await?.sync().await?);
    }

    Ok(dir)
}

/// Return a vector of all tests
fn all_tests<'q>(
    client: &'q Query,
    arguments: &'q TestCommand,
) -> Vec<(
    &'static str,
    Pin<Box<dyn Future<Output = Result<Directory>> + 'q>>,
    bool,
)> {
    let all_tests: Vec<(&str, Pin<Box<dyn Future<Output = _>>>, bool)> = vec![
        // (name, future, is_sequential)
        ("rustfmt", Box::pin(crate::targets::rustfmt(client)), false),
        ("typos", Box::pin(crate::targets::typos(client)), false),
        (
            "native_tests",
            Box::pin(crate::targets::native_tests(client)),
            false,
        ),
        (
            "wasm_unit",
            Box::pin(crate::targets::wasm_unit_tests(client)),
            false,
        ),
        (
            "clippy_workspace",
            Box::pin(crate::targets::clippy_workspace(client)),
            false,
        ),
        (
            "clippy_natrix",
            Box::pin(crate::targets::clippy_natrix(client)),
            false,
        ),
        (
            "test_docs",
            Box::pin(crate::targets::test_docs(client)),
            false,
        ),
        (
            "cargo_deny_natrix",
            Box::pin(crate::targets::cargo_deny(client, "./crates/natrix")),
            false,
        ),
        (
            "cargo_deny_natrix_cli",
            Box::pin(crate::targets::cargo_deny(client, "./crates/natrix-cli")),
            false,
        ),
        (
            "unused_deps",
            Box::pin(crate::targets::unused_deps(client)),
            false,
        ),
        (
            "outdated_deps",
            Box::pin(crate::targets::outdated_deps(client)),
            false,
        ),
        (
            "test_project_gen_stable",
            Box::pin(crate::targets::test_project_gen(client, "stable")),
            false,
        ),
        (
            "test_project_gen_nightly",
            Box::pin(crate::targets::test_project_gen(client, "nightly")),
            false,
        ),
        (
            "test_book_links",
            Box::pin(crate::targets::test_book_links(client)),
            true,
        ),
        (
            "test_book_examples",
            Box::pin(crate::targets::test_book_examples(client)),
            true,
        ),
        (
            "integration_test_dev",
            Box::pin(crate::targets::integration_test(
                client,
                IntegrationTestMode::Dev,
                arguments,
            )),
            true,
        ),
        (
            "integration_test_release",
            Box::pin(crate::targets::integration_test(
                client,
                IntegrationTestMode::Release,
                arguments,
            )),
            true,
        ),
        (
            "integration_test_build",
            Box::pin(crate::targets::integration_test(
                client,
                IntegrationTestMode::Build,
                arguments,
            )),
            true,
        ),
    ];
    all_tests
}

/// Generate the final report
pub fn generate_report(
    client: &Query,
    reports: Directory,
    arguments: &TestCommand,
) -> Result<Directory> {
    let result = client
        .container()
        .from("andgineer/allure")
        .with_directory("/reports", reports)
        .with_mounted_cache(
            "/history-cache/",
            client.cache_volume(format!("allure-history-{}", arguments.filter.join(","))),
        )
        .with_exec_opts(
            vec!["mv", "-v", "/history-cache/history", "/reports/history"],
            ContainerWithExecOptsBuilder::default()
                .expect(ReturnType::Any)
                .build()?,
        )
        .with_exec(vec![
            "allure",
            "generate",
            "--name",
            "Natrix CI",
            "/reports",
        ])
        .with_exec(vec![
            "mv",
            "-v",
            "/allure-report/history",
            "/history-cache/history",
        ])
        .directory("/allure-report");

    Ok(result)
}

/// Serve the report
pub async fn serve_dist(client: &Query, dist: Directory) -> Result<()> {
    let container = client
        .container()
        .from("nginx:alpine")
        .with_directory("/usr/share/nginx/html", dist)
        .with_exposed_port(80);
    container.sync().await?;
    let service = container.as_service();

    let server = async move {
        let res = service
            .up_opts(ServiceUpOpts {
                ports: Some(vec![PortForward {
                    backend: 80,
                    frontend: 8000,
                    protocol: dagger_sdk::NetworkProtocol::Tcp,
                }]),
                random: None,
            })
            .await;
        if res.is_err() {
            service
                .up_opts(ServiceUpOpts {
                    ports: None,
                    random: Some(true),
                })
                .await?;
        }

        Ok::<(), eyre::Report>(())
    };

    let handle = tokio::spawn(server);
    println!("=========== RESULT GENERATED ============");
    println!("VISIT ABOVE LINK TO SEE");
    let _ = handle.await;

    Ok(())
}
