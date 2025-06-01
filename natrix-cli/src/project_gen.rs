//! Generate project

use std::fs;
use std::path::{Path, PathBuf};

use crate::prelude::*;

/// Flag for extracting css
pub const FEATURE_EXTRACT_CSS: &str = "__natrix_internal_extract_css";
///
/// Flag for runtime css
pub const FEATURE_RUNTIME_CSS: &str = "__natrix_internal_runtime_css";

/// Generate a new project
pub(crate) fn generate_project(name: &str, stable: bool) -> std::result::Result<(), anyhow::Error> {
    let root = PathBuf::from(name);
    fs::create_dir_all(&root)?;

    let nightly = !stable;

    // we assume the library version is the same as the cli version
    // This means that even if the cli isnt modified it should publish new versions along with the
    // library
    let natrix_version = env!("CARGO_PKG_VERSION");

    let mut natrix_table = format!(r#"version = "{natrix_version}""#);
    if let Ok(path) = std::env::var("NATRIX_PATH") {
        natrix_table = format!(r#"{natrix_table}, path = "{path}""#);
    }
    let natrix_test_table = format!(r#"natrix = {{{natrix_table}, features=["test_utils"]}}"#);

    let mut features = vec!["default_app"];

    if nightly {
        features.push("nightly");
    }
    let features = features
        .into_iter()
        .map(|feat| format!(r#""{feat}""#))
        .collect::<Vec<_>>()
        .join(",");
    natrix_table = format!(r"{natrix_table}, features = [{features}]");

    let natrix_decl = format!("natrix = {{ {natrix_table} }}");
    let natrix_decl = natrix_decl.trim();

    let cargo_toml = format!(
        r#"
[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[dependencies]
{natrix_decl}
log = {{version = "0.4", features = ["max_level_warn", "release_max_level_off"]}}

[dev-dependencies]
{natrix_test_table}
wasm-bindgen-test = "0.3.50"

[profile.release]
opt-level = "z"
codegen-units = 1
lto = "fat"
panic = "abort"
strip = "symbols"

[features]
# IMPORTANT: 
# These are feature forwards for internal compilation flags used by the bundler
# DO NOT RENAME OR REMOVE THESE
{FEATURE_EXTRACT_CSS} = ["natrix/_internal_extract_css"]
{FEATURE_RUNTIME_CSS} = ["natrix/_internal_runtime_css"]
        "#
    );
    fs::write(root.join("Cargo.toml"), cargo_toml)?;

    let gitignore = "
target
dist
    "
    .trim();
    fs::write(root.join(".gitignore"), gitignore)?;

    generate_main_rs(&root, name, nightly)?;

    std::process::Command::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(&root)
        .status()?;

    println!(
        "✨ {} {}",
        "Project created".bright_green(),
        root.display().cyan()
    );
    println!(
        "{}",
        "Run `natrix dev` to start the dev server".bright_blue()
    );

    Ok(())
}

/// Generate the main.rs file for a new project
fn generate_main_rs(root: &Path, name: &str, nightly: bool) -> Result<(), anyhow::Error> {
    let nightly_lints = if nightly {
        "
// This is a nightly only feature to warn you when you are passing certain types across a
// `await` boundary, `natrix` marks multiple types with this attribute to prevent you from
// causing runtime panics.
#![feature(must_not_suspend)]
#![warn(must_not_suspend)]
        "
        .trim()
    } else {
        ""
    };
    let nightly_associated_types_are_optional = if nightly {
        ""
    } else {
        "
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
"
        .trim()
    };
    let main_rs = format!(
        r#"
{nightly_lints}
// Panicking in a wasm module will cause the state to be invalid
// And it might cause UB on the next event handler execution.
// (By default natrix uses a panic hook that blocks further event handler calls after a panic)
#![deny(clippy::panic, clippy::unwrap_used, clippy::expect_used)]
// These are more strict anti panic lints that you might want to enable
// #![warn(clippy::arithmetic_side_effects, clippy::indexing_slicing, clippy::unreachable)]

use natrix::prelude::*;

#[derive(Component)]
struct HelloWorld;

impl Component for HelloWorld {{
    {nightly_associated_types_are_optional}
    fn render() -> impl Element<Self> {{
        e::h1().text("Hello {name}").id("HELLO")
    }}
}}

fn main() {{
    natrix::mount(HelloWorld);
}}

#[cfg(test)]
mod tests {{
    use super::*;
    use natrix::test_utils;

    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test() {{
        test_utils::mount_test(HelloWorld);
        let element = test_utils::get("HELLO");
        assert_eq!(element.text_content(), Some("Hello {name}".to_string()));
    }}
}}
"#
    );
    let src = root.join("src");
    fs::create_dir_all(&src)?;
    fs::write(src.join("main.rs"), main_rs)?;

    let channel = if nightly { "nightly" } else { "stable" };

    let components = if nightly {
        r#"components = ["rust-src"]"#
    } else {
        ""
    };

    let toolchain_toml = format!(
        r#"
[toolchain]
channel = "{channel}"
targets = ["wasm32-unknown-unknown"]
{components}
        "#
    );
    fs::write(root.join("rust-toolchain.toml"), toolchain_toml)?;

    Ok(())
}
