//! Build the wasm and js of a file

use std::path::{Path, PathBuf};
use std::{fs, process};

use super::{BINDGEN_OUTPUT_NAME, MACRO_OUTPUT_DIR};
use crate::options::BuildProfile;
use crate::prelude::*;
use crate::project_gen::FEATURE_RUNTIME_CSS;
use crate::{options, utils};

/// Run wasmbindgen to generate the clue
pub(crate) fn wasm_bindgen(
    config: &options::BuildConfig,
    wasm_file: &PathBuf,
) -> Result<(PathBuf, PathBuf)> {
    let mut command = process::Command::new("wasm-bindgen");
    command
        .arg(wasm_file)
        .args(["--out-dir", &utils::path_str(&config.dist)])
        .args(["--target", "web"])
        .args(["--out-name", BINDGEN_OUTPUT_NAME])
        .arg("--no-typescript")
        .arg("--omit-default-module-path");
    if config.profile == options::BuildProfile::Dev {
        command.arg("--debug").arg("--keep-debug");
    } else {
        command
            .arg("--remove-name-section")
            .arg("--remove-producers-section")
            .arg("--no-demangle");
    }
    utils::run_with_spinner(command, utils::create_spinner("✍️ wasm_bindgen")?)?;

    let js_file = config.dist.join(format!("{BINDGEN_OUTPUT_NAME}.js"));
    if config.profile == options::BuildProfile::Release {
        minimize_js(&js_file)?;
    }

    Ok((
        config.dist.join(format!("{BINDGEN_OUTPUT_NAME}_bg.wasm")),
        js_file,
    ))
}

/// Minimize the given js file
pub(crate) fn minimize_js(js_file: &PathBuf) -> Result<(), anyhow::Error> {
    let spinner = utils::create_spinner("🗜️ Minimizing JS")?;

    let js_code = fs::read_to_string(js_file)?;
    let allocator = oxc::allocator::Allocator::new();
    let parser = oxc::parser::Parser::new(&allocator, &js_code, oxc::span::SourceType::cjs());

    let mut program = parser.parse().program;
    let minifier = oxc::minifier::Minifier::new(oxc::minifier::MinifierOptions {
        mangle: Some(oxc::minifier::MangleOptions {
            top_level: true,
            ..Default::default()
        }),
        compress: Some(oxc::minifier::CompressOptions {
            drop_console: !utils::is_feature_enabled("keep_console_in_release", false)?,
            drop_debugger: true,
            ..Default::default()
        }),
    });
    let symbols = minifier.build(&allocator, &mut program).scoping;

    let codegen = oxc::codegen::Codegen::new()
        .with_options(oxc::codegen::CodegenOptions {
            minify: true,
            comments: false,
            ..Default::default()
        })
        .with_scoping(symbols);
    let js_code = codegen.build(&program).code;
    std::fs::write(js_file, js_code)?;

    spinner.finish();
    Ok(())
}

/// Build the project wasm
pub(crate) fn build_wasm(config: &options::BuildConfig) -> Result<PathBuf> {
    let rustc_version_meta = rustc_version::version_meta()?;
    let rustc_is_nightly = rustc_version_meta.channel == rustc_version::Channel::Nightly;

    let invalidate = if config.invalidate_cache {
        let time = std::time::SystemTime::now();
        let eleapsed_time = time.duration_since(std::time::UNIX_EPOCH)?;
        eleapsed_time.as_secs()
    } else {
        0
    };

    let settings = natrix_shared::macros::Settings {
        output_dir: config.temp_dir.join(MACRO_OUTPUT_DIR),
        base_path: config.base_path.to_string(),
        invalidate,
    };
    let settings = natrix_shared::macros::bincode::encode_to_vec(
        settings,
        natrix_shared::macros::bincode_config(),
    )?;
    let settings = data_encoding::BASE64_NOPAD.encode(&settings);

    let mut command = process::Command::new("cargo");
    command
        .arg("build")
        .args(["--color", "always"])
        .args(["--target", "wasm32-unknown-unknown"])
        .args(["--profile", config.profile.cargo()])
        .env(natrix_shared::MACRO_SETTINGS, settings);

    match config.profile {
        BuildProfile::Release => {
            let mut rustc_flags = String::from("-C target-feature=+bulk-memory,+reference-types ");
            if rustc_is_nightly {
                let std_features = String::from("optimize_for_size");
                command
                    .args(["-Z", "build-std=core,std,panic_abort"])
                    .arg(format!("-Zbuild-std-features={std_features}"));
                rustc_flags.push_str("-Zfmt-debug=none -Zlocation-detail=none -Zshare-generics=y");
            } else {
                println!(
                        "{}",
                        "⚠️ Using stable rust, nightly rust allows for better optimizations and smaller wasm files"
                            .yellow()
                            .bold()
                    );
            }
            command.env("RUSTFLAGS", rustc_flags);
        }
        BuildProfile::Dev => {
            command.args(["--features", FEATURE_RUNTIME_CSS]);
        }
    }
    utils::run_with_spinner(command, utils::create_spinner("⚙️ wasm")?).context("Running cargo")?;

    find_wasm(config).context("Finding wasm file")
}

/// Return the path to the first wasm file in the folder
pub(crate) fn find_wasm(config: &options::BuildConfig) -> Result<PathBuf> {
    let target = utils::find_target()?;
    let name = utils::get_project_name()?;
    let target = target
        .join("wasm32-unknown-unknown")
        .join(config.profile.target());

    if let Some(wasm) = search_dir_for_wasm(&target, &name)? {
        return Ok(wasm);
    }

    if let Some(wasm) = search_dir_for_wasm(&target.join("deps"), &name)? {
        return Ok(wasm);
    }

    Err(anyhow!("Wasm file not found in {}", target.display()))
}

/// Search the given directory for a wasm file
pub(crate) fn search_dir_for_wasm(
    target: &Path,
    name: &str,
) -> Result<Option<PathBuf>, anyhow::Error> {
    let expected_file_name = format!("{name}.wasm");
    for file in target.read_dir()?.flatten() {
        let path = file.path();
        if let Some(name) = path.file_name() {
            let name = name.to_string_lossy();
            if name == expected_file_name {
                return Ok(Some(file.path()));
            }
        }
    }
    Ok(None)
}

/// Optimize the given wasm file
pub(crate) fn optimize_wasm(wasm_file: &PathBuf) -> Result<(), anyhow::Error> {
    let spinner = utils::create_spinner("🔎 Optimize wasm")?;

    let mut command = process::Command::new("wasm-opt");
    command
        .arg(wasm_file)
        .arg("-o")
        .arg(wasm_file)
        .arg("--all-features")
        .arg("--strip-debug")
        .arg("--strip-dwarf")
        .arg("--strip-producers")
        .arg("--strip-target-features");
    command.args(["--converge", "-Oz"]);

    let result = command.status()?.success();

    spinner.finish();
    if !result {
        return Err(anyhow!("Failed to optimize"));
    }
    Ok(())
}

/// Get the strings from a wasm file
pub(crate) fn get_wasm_strings(wasm_file: &Path) -> Result<Vec<String>> {
    let wasm_bytes = fs::read(wasm_file)?;
    let mut strings = Vec::new();

    let parser = wasmparser::Parser::new(0);
    for payload in parser.parse_all(&wasm_bytes) {
        if let wasmparser::Payload::DataSection(data_section_reader) = payload? {
            for data in data_section_reader {
                let data = data?;
                if let Some(bytes) = data.data.get(0..) {
                    if let Ok(string) = std::str::from_utf8(bytes) {
                        strings.push(string.to_string());
                    } else {
                        // Clean out problematic bytes
                        let cleaned = bytes
                            .iter()
                            .filter(|&&x| x.is_ascii())
                            .copied()
                            .collect::<Vec<u8>>();
                        if let Ok(string) = std::str::from_utf8(&cleaned) {
                            strings.push(string.to_string());
                        } else {
                            return Err(anyhow!(
                                "Failed to extract string from wasm, this might lead to wrongful DCE optimization"
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok(strings)
}
