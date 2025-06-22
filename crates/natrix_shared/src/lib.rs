//! Shared constants, functions, etc across the natrix project.

/// The mount point for the auto generated `index.html` from the cli.
pub const MOUNT_POINT: &str = "NATRIX_MOUNT";

/// The env var for setting macro settings
pub const MACRO_SETTINGS: &str = "NATRIX_MACRO_SETTINGS";

/// Code used for macros and bundler
#[cfg(feature = "macros")]
pub mod macros {
    pub use bincode;

    /// The asset format
    // IMPORTANT: Macro assumes encoding this cant fail
    // <https://docs.rs/bincode/latest/bincode/error/enum.EncodeError.html>
    #[derive(bincode::Decode, bincode::Encode)]
    pub enum MacroEmisson {
        /// An asset that needs bundling
        Asset {
            /// The file path to the asset
            path: std::path::PathBuf,
            /// The emitted name for the asset
            emitted_path: String,
        },
    }

    /// The settings for the macros
    #[derive(bincode::Decode, bincode::Encode)]
    pub struct Settings {
        /// The output dir for the macros
        pub output_dir: std::path::PathBuf,
        /// The base path for the macros
        pub base_path: String,
        /// A attribute to be used for invalidating the macro outputs
        pub invalidate: u64,
    }

    /// the  bincode config to use
    #[must_use]
    pub fn bincode_config() -> impl bincode::config::Config {
        bincode::config::standard()
    }
}
