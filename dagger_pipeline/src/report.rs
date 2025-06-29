//! Construct CI report.

use dagger_sdk::{ContainerWithExecOptsBuilder, ReturnType};

use crate::prelude::*;

/// Generate the final report
pub async fn generate_report(client: &Query, state: &GlobalState) -> Result<()> {
    let (native_tests, wasm_unit_nightly, wasm_unit_stable) = tokio::try_join!(
        crate::targets::native_tests(client, state),
        crate::targets::wasm_unit_tests(client, state, "nightly"),
        crate::targets::wasm_unit_tests(client, state, "stable")
    )?;

    client
        .container()
        .from("andgineer/allure")
        .with_directory("/reports", wasm_unit_nightly)
        .with_directory("/reports", wasm_unit_stable)
        .with_file("/reports/native_tests.xml", native_tests)
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
        .directory("/allure-report")
        .export("./allure-report")
        .await?;

    Ok(())
}
