//! Shared constants, functions, etc across the natrix project.

/// The mount point for the auto generated `index.html` from the cli.
pub const MOUNT_POINT: &str = "NATRIX_MOUNT";

/// The env var to use to pass the output dir to macros.
pub const MACRO_OUTPUT_ENV: &str = "NATRIX_MACRO_OUTPUT";

/// The env var used to invalidate the macro outputs.
pub const MACRO_INVALIDATE_ENV: &str = "NATRIX_MACRO_INVALIDATE";

/// The env var used to indicate the base url to the macros
pub const MACRO_BASE_PATH_ENV: &str = "NATRIX_BASE_PATH";

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
