//! Build system and project generator for natrix

use clap::Parser;

/// Reusable imports
mod prelude {
    pub use anyhow::{Context, Result, anyhow};
    pub use owo_colors::OwoColorize;
}

use prelude::*;

mod build;
mod dev_server;
mod options;
mod project_gen;
mod utils;

// TODO: Disable colors in non-interactive terminals.

fn main() -> Result<()> {
    let cli = options::Cli::parse();

    match cli {
        options::Cli::New { name, stable } => project_gen::generate_project(&name, stable),
        options::Cli::Dev(args) => dev_server::do_dev(&args),
        options::Cli::Build(args) => {
            build::build(&args.into_build_config()?).context("Building application")?;
            Ok(())
        }
    }
}
