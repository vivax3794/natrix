//! Various fixes that will be exported back to the host.

use crate::base_images::binstall;
use crate::prelude::*;

/// Apply rustfmt fixes
pub fn fmt(client: &Query, source: Directory) -> Directory {
    crate::base_images::rust(client)
        .with_directory("/app", source)
        .with_workdir("/app")
        .with_exec(vec!["cargo", "+nightly", "fmt"])
        .directory("/app")
}

/// Apply typo fixes
pub fn typos(client: &Query, source: Directory) -> Result<Directory> {
    Ok(crate::base_images::base(client)
        .with_directory("/bin", binstall(client, "typos-cli"))
        .with_directory("/app", source)
        .with_workdir("/app")
        .with_exec_any(vec!["typos", "-w"])?
        .directory("/app"))
}

/// Update snapshots
pub fn snapshots(client: &Query, source: Directory) -> Directory {
    crate::base_images::rust(client)
        .with_directory("/bin", binstall(client, "cargo-insta"))
        .with_directory("/bin", binstall(client, "cargo-nextest"))
        .with_directory("/app", source)
        .with_mounted_cache("/app/target", client.cache_volume("rust-target"))
        .with_workdir("/app/crates/natrix")
        .with_exec(vec![
            "cargo",
            "+nightly",
            "insta",
            "test",
            "--accept",
            "--unreferenced",
            "delete",
            "--test-runner",
            "nextest",
            "--all-features",
        ])
        .directory("/app")
}
