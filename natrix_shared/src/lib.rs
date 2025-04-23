//! Shared constants, functions, etc across the natrix project.

#![forbid(
    unsafe_code,
    clippy::todo,
    clippy::unreachable,
    clippy::unwrap_used,
    clippy::unreachable,
    clippy::indexing_slicing
)]
#![deny(
    clippy::dbg_macro,
    clippy::expect_used,
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason,
    clippy::arithmetic_side_effects
)]
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::pedantic,
    unfulfilled_lint_expectations
)]

/// The mount point for the auto generated `index.html` from the cli.
pub const MOUNT_POINT: &str = "NATRIX_MOUNT";

/// The env var to use to pass the output dir to macros.
pub const MACRO_OUTPUT_ENV: &str = "NATRIX_MACRO_OUTPUT";

/// The env var used to invalidate the macro outputs.
pub const MACRO_INVALIDATE_ENV: &str = "NATRIX_MACRO_INVALIDATE";

/// The asset format
#[cfg(feature = "assets")]
#[derive(bincode::Decode, bincode::Encode)]
pub struct Asset {
    /// The file path to the asset
    pub path: std::path::PathBuf,
    /// The emitted name for the asset
    pub emitted_path: String,
}

#[cfg(feature = "assets")]
pub use bincode;

/// the  bincode config to use
#[cfg(feature = "assets")]
#[must_use]
pub fn bincode_config() -> impl bincode::config::Config {
    bincode::config::standard()
}
