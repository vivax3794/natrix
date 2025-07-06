//! Construct CI report.

use std::pin::Pin;

use dagger_sdk::{ContainerWithExecOptsBuilder, PortForward, ReturnType, ServiceUpOpts};
use futures::{StreamExt, stream};

use crate::TestCommand;
use crate::prelude::*;
use crate::targets::IntegrationTestMode;

/// Run all tests and return a directory of allure reports
pub async fn run_all_tests(client: &Query, arguments: TestCommand) -> Result<Directory> {
    let mut tasks: Vec<Pin<Box<dyn Future<Output = _>>>> = vec![
        Box::pin(crate::targets::native_tests(client)),
        Box::pin(crate::targets::wasm_unit_tests(client, "nightly")),
        Box::pin(crate::targets::clippy_workspace(client)),
        Box::pin(crate::targets::rustfmt(client)),
        Box::pin(crate::targets::typos(client)),
        Box::pin(crate::targets::test_docs(client)),
        Box::pin(crate::targets::cargo_deny(client, "./crates/natrix")),
    ];
    let mut need_sequential: Vec<Pin<Box<dyn Future<Output = _>>>> = vec![];

    if !arguments.quick {
        tasks.extend::<[Pin<Box<dyn Future<Output = _>>>; _]>([
            Box::pin(crate::targets::unused_deps(client)),
            Box::pin(crate::targets::clippy_natrix(client, "nightly")),
            Box::pin(crate::targets::clippy_natrix(client, "stable")),
            Box::pin(crate::targets::wasm_unit_tests(client, "stable")),
            Box::pin(crate::targets::cargo_deny(client, "./crates/natrix-cli")),
            Box::pin(crate::targets::outdated_deps(client)),
            Box::pin(crate::targets::test_project_gen(client, "stable")),
            Box::pin(crate::targets::test_project_gen(client, "nightly")),
        ]);
        need_sequential.extend::<[Pin<Box<dyn Future<Output = _>>>; _]>([
            Box::pin(crate::targets::test_book_links(client)),
            Box::pin(crate::targets::test_book_examples(client)),
            Box::pin(crate::targets::integration_test(
                client,
                IntegrationTestMode::Dev,
                &arguments,
            )),
            Box::pin(crate::targets::integration_test(
                client,
                IntegrationTestMode::Release,
                &arguments,
            )),
            Box::pin(crate::targets::integration_test(
                client,
                IntegrationTestMode::Build,
                &arguments,
            )),
        ]);
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

/// Generate the final report
pub fn generate_report(client: &Query, reports: Directory) -> Result<Directory> {
    let result = client
        .container()
        .from("andgineer/allure")
        .with_directory("/reports", reports)
        .with_mounted_cache("/history-cache/", client.cache_volume("allure-history"))
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
pub async fn serve_report(client: &Query, report: Directory) -> Result<()> {
    let container = client
        .container()
        .from("nginx:alpine")
        .with_directory("/usr/share/nginx/html", report)
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
    println!("=========== REPORT GENERATED ============");
    println!("VISIT ABOVE LINK TO SEE");
    let _ = handle.await;

    Ok(())
}
