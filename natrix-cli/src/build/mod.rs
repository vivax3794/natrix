//! Build a natrix project

use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::options::BuildProfile;
use crate::prelude::*;
use crate::{options, utils};

pub(crate) mod assets;
mod css;
mod wasm_js;

/// The directory to store macro outputs
const MACRO_OUTPUT_DIR: &str = "macro";
/// The name of the js file
const BINDGEN_OUTPUT_NAME: &str = "code";
/// Name of the collected css
const CSS_OUTPUT_NAME: &str = "styles.css";

/// Build a project
pub(crate) fn build(config: &options::BuildConfig) -> Result<assets::AssetManifest> {
    println!("🧹 {}", "Cleaning dist".bright_black(),);
    let _ = fs::remove_dir_all(&config.dist);

    if config.invalidate_cache {
        let _ = fs::remove_dir_all(&config.temp_dir);
    }
    let _ = fs::create_dir_all(config.temp_dir.join(MACRO_OUTPUT_DIR));

    println!(
        "🚧 {} (using profile {})",
        "Starting Build".bright_blue(),
        config.profile.readable().cyan()
    );
    std::fs::create_dir_all(&config.dist).context("Creating dist")?;

    let source_wasm_file = wasm_js::build_wasm(config).context("Building wasm")?;
    let (wasm_file, js_file) = wasm_js::wasm_bindgen(config, &source_wasm_file)?;
    if config.profile == options::BuildProfile::Release {
        wasm_js::optimize_wasm(&wasm_file)?;
    }

    let wasm_file = cache_bust_file(config, wasm_file)?;
    let js_file = cache_bust_file(config, js_file)?;

    let asset_manifest = assets::collect_macro_output(config)?;

    let css_file = if config.ssg {
        let css_file = css::collect_css(config, &wasm_file)?;
        let css_file = cache_bust_file(config, css_file)?;
        Some(css_file)
    } else {
        None
    };

    generate_html(config, &wasm_file, &js_file, css_file)?;

    println!(
        "📦 {} {}",
        "Result in".bright_blue(),
        config.dist.display().cyan()
    );

    Ok(asset_manifest)
}

/// Generate the html file to be used
pub(crate) fn generate_html(
    config: &options::BuildConfig,
    wasm_file: &Path,
    js_file: &Path,
    css_file: Option<PathBuf>,
) -> Result<()> {
    let base_path = &config.base_path;

    let html_file = config.dist.join("index.html");
    let js_file = utils::get_filename(js_file)?;
    let wasm_file = utils::get_filename(wasm_file)?;

    let style_link = if let Some(css_file) = css_file {
        let css_file = utils::get_filename(&css_file)?;
        format!(r#"<link rel="stylesheet" href="{base_path}/{css_file}"/>"#)
    } else {
        String::new()
    };

    let js_reload = if let Some(port) = config.live_reload {
        format!(
            r#"
            const reload_ws = new WebSocket("ws://localhost:{port}");
            reload_ws.onmessage = (event) => {{
                location.reload();
            }};
            "#
        )
    } else {
        String::new()
    };

    let content = format!(
        r#"
<!doctype html>
<html>
    <head>
        {style_link}
    </head>
    <body>
        <div id="{}">This website requires Javascript and Wasm support.</div>
        <script type="module">
            import init from "{base_path}/{js_file}";
            init("{base_path}/{wasm_file}");
            {js_reload}
        </script>
    </body>
</html>
    "#,
        natrix_shared::MOUNT_POINT,
    );

    std::fs::write(html_file, content.trim())?;

    Ok(())
}

/// Moves the given file to a new location in accordane with cache busting options
/// Returns the new file location
pub(crate) fn cache_bust_file(
    config: &options::BuildConfig,
    original_file: PathBuf,
) -> Result<PathBuf> {
    let Some(original_filename) = original_file.file_name() else {
        return Ok(original_file);
    };
    let original_filename = original_filename.to_string_lossy();

    let new_filename = match config.cache_bust {
        options::CacheBustOption::None => original_filename.into_owned(),
        options::CacheBustOption::Timestamp => {
            let current_time = std::time::SystemTime::now();
            let since_epoch = current_time.duration_since(std::time::UNIX_EPOCH)?;
            let unix_time_stamp = since_epoch.as_secs();
            let encoded_timestamp =
                data_encoding::BASE64URL_NOPAD.encode(&unix_time_stamp.to_le_bytes());
            format!("{encoded_timestamp}-{original_filename}")
        }
        options::CacheBustOption::Content => {
            let content = fs::read(&original_file)?;
            let mut hasher = DefaultHasher::default();
            content.hash(&mut hasher);
            let hash = hasher.finish();

            let encoded_hash = data_encoding::BASE64URL_NOPAD.encode(&hash.to_le_bytes());
            format!("{encoded_hash}-{original_filename}")
        }
    };

    let new_file = original_file.with_file_name(new_filename);

    fs::rename(original_file, &new_file)?;
    Ok(new_file)
}
