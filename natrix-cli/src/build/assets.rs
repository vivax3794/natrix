//! Assets bundling and optimization

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::{MACRO_OUTPUT_DIR, options, utils};
use crate::prelude::*;

/// Describes the translation from asset paths to wanted url
#[derive(Default)]
pub(crate) struct AssetManifest {
    /// The actual mapping
    pub(crate) mapping: HashMap<String, PathBuf>,
}

/// Collect the outputs of the macros
pub(crate) fn collect_macro_output(config: &options::BuildConfig) -> Result<AssetManifest> {
    let mut asset_files = Vec::new();

    for file in get_macro_output_files(config)? {
        let extension = file.extension().map(|ext| ext.to_string_lossy());
        match extension.as_ref().map(AsRef::as_ref) {
            Some("asset") => asset_files.push(file),
            _ => return Err(anyhow!("Invalid file extension found in macro output")),
        }
    }

    let manifest = collect_asset_manifest(asset_files)?;

    if !config.should_direct_serve_files() {
        copy_assets_to_dist(config, &manifest)?;
    }

    Ok(manifest)
}

/// Copy asset manifest to dist
pub(crate) fn copy_assets_to_dist(
    config: &options::BuildConfig,
    manifest: &AssetManifest,
) -> Result<()> {
    let spinner = utils::create_spinner("📂 Copying Assets")?;
    for (wanted_url, file) in &manifest.mapping {
        let target_file = config.dist.join(wanted_url);
        if let Err(err) = fs::copy(file, target_file) {
            spinner.finish();
            return Err(err.into());
        }
    }

    spinner.finish();
    Ok(())
}

/// Collect the `.asset` files into a asset manifest
pub(crate) fn collect_asset_manifest(asset_files: Vec<PathBuf>) -> Result<AssetManifest> {
    let spinner = utils::create_spinner("📋 Parsing Asset Manifest")?;

    let mut mapping = HashMap::with_capacity(asset_files.len());
    for file in asset_files {
        let mut file_reader = fs::File::open(file)?;
        let asset: natrix_shared::macros::Asset =
            natrix_shared::macros::bincode::decode_from_std_read(
                &mut file_reader,
                natrix_shared::macros::bincode_config(),
            )?;
        mapping.insert(asset.emitted_path, asset.path);
    }

    spinner.finish();
    Ok(AssetManifest { mapping })
}

/// Get all files in the sub folders of `MACRO_OUTPUT_DIR`
pub(crate) fn get_macro_output_files(
    config: &options::BuildConfig,
) -> Result<impl Iterator<Item = PathBuf>> {
    Ok(fs::read_dir(config.temp_dir.join(MACRO_OUTPUT_DIR))?
        .flatten()
        .flat_map(|folder| fs::read_dir(folder.path()).into_iter().flatten().flatten())
        .map(|entry| entry.path()))
}
