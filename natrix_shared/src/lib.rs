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
