//! Build system and project generator for natrix
#![forbid(unsafe_code)]
#![deny(
    clippy::todo,
    clippy::unreachable,
    clippy::unwrap_used,
    clippy::unreachable,
    clippy::indexing_slicing,
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

use std::borrow::Cow;
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;
use std::{fs, process, thread};

use anyhow::{Context, Result, anyhow};
use clap::{Parser, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use notify::Watcher;
use owo_colors::OwoColorize;
use tiny_http::{Header, Response, Server};

/// The directory to store macro outputs
const MACRO_OUTPUT_DIR: &str = "macro";
/// The name of the js file
const BINDGEN_OUTPUT_NAME: &str = "code";
/// Name of the collected css
const CSS_OUTPUT_NAME: &str = "styles.css";

/// Find the closet target folder
fn find_target(mode: BuildProfile) -> Result<PathBuf> {
    let mut current = PathBuf::from(".").canonicalize()?;
    loop {
        let target = current.join("target");
        if target.exists() {
            return Ok(target.join(format!("natrix-{}", mode.redable())));
        }
        if let Some(parent) = current.parent() {
            current = parent.to_owned();
        } else {
            return Err(anyhow!("Target not found"));
        }
    }
}

/// Natrix CLI
#[derive(Parser)]
enum Cli {
    /// Spawn a dev server
    Dev(BuildConfigArgs),
    /// Build the project
    Build(BuildConfigArgs),
}

/// Settings for building the server
#[derive(Parser)]
struct BuildConfigArgs {
    /// Build profile to use
    #[arg(short, long, value_enum)]
    profile: Option<BuildProfile>,
    /// Location to output build files
    #[arg(short, long)]
    dist: Option<PathBuf>,
}

/// Settings for building the server
struct BuildConfig {
    /// Build profile to use
    profile: BuildProfile,
    /// Location to putput build files
    dist: PathBuf,
    /// Location for the temp dir
    temp_dir: PathBuf,
}

impl BuildConfigArgs {
    /// Replace optional arguments (that have defaults) with the defaults for the `dev` subcommand
    fn fill_build_defaults(self) -> Result<BuildConfig> {
        let profile = self.profile.unwrap_or(BuildProfile::Release);
        Ok(BuildConfig {
            profile,
            dist: self.dist.unwrap_or_else(|| PathBuf::from("./dist")),
            temp_dir: find_target(profile)?,
        })
    }

    /// Replace optional arguments (that have defaults) with the defaults for the `build` subcommand
    fn fill_dev_defaults(self) -> Result<BuildConfig> {
        let profile = self.profile.unwrap_or(BuildProfile::Dev);
        let target = find_target(profile)?;

        let dist = if let Some(dist) = self.dist {
            dist
        } else {
            target.join("dist")
        };

        Ok(BuildConfig {
            profile,
            dist,
            temp_dir: target,
        })
    }
}

/// Build profile
#[derive(Clone, Copy, ValueEnum, PartialEq, Eq)]
enum BuildProfile {
    /// Runs with optimizations
    Release,
    /// Does not do any optimization
    Dev,
}

impl BuildProfile {
    /// Return a more redable version of this profile name
    fn redable(self) -> &'static str {
        match self {
            Self::Release => "release",
            Self::Dev => "dev",
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::Dev(config) => do_dev(config),
        Cli::Build(config) => build(&config.fill_build_defaults()?).context("Building application"),
    }
}

/// Do the dev server
fn do_dev(config: BuildConfigArgs) -> Result<()> {
    let config = config.fill_dev_defaults()?;

    let (tx_notify, rx_notify) = mpsc::channel();
    let (tx_reload, rx_reload) = mpsc::channel();

    let mut watcher = notify::recommended_watcher(tx_notify)?;
    watcher.watch(&PathBuf::from("."), notify::RecursiveMode::Recursive)?;

    if let Err(err) = build(&config) {
        println!("{}", err.red());
    }

    let dist = config.dist.clone();
    thread::spawn(|| spawn_server(dist, rx_reload));

    loop {
        let event = rx_notify.recv()??;

        if event.kind.is_modify() {
            if let Err(err) = build(&config) {
                println!("{}", err.red());
            } else {
                tx_reload.send(())?;
            }
            while rx_notify.try_recv().is_ok() {}
        }
    }
}

/// Spawn a dev server for doing reloading
#[expect(
    clippy::expect_used,
    clippy::needless_pass_by_value,
    reason = "This is running in a thread"
)]
fn spawn_server(folder: PathBuf, reload_signal: Receiver<()>) {
    let server = Server::http("0.0.0.0:8000").expect("Failed to start server");

    let mut should_reload = false;

    for request in server.incoming_requests() {
        if reload_signal.try_recv().is_ok() {
            should_reload = true;
        }

        let url = request.url();
        let path = if url == "/" {
            folder.join("index.html")
        } else if url == "/RELOAD" {
            if should_reload {
                let response = Response::from_string("RELOAD NOW");
                let _ = request.respond(response);
                should_reload = false;
                continue;
            }
            let response = Response::from_string("NO RELOAD").with_status_code(404);
            let _ = request.respond(response);
            continue;
        } else {
            folder.join(url.strip_prefix("/").unwrap_or(url))
        };
        if url.contains("..") {
            let response = Response::from_string("PATH TRAVERSAL DETECTED").with_status_code(404);
            let _ = request.respond(response);
            println!(
                "{}",
                "Path traversal detected in URL, terminating server for security."
                    .bold()
                    .red()
                    .on_black()
            );
            return;
        }

        let response = if path.exists() && path.is_file() {
            let content_type: &[u8] = match path.extension().and_then(|x| x.to_str()) {
                Some("html") => b"text/html",
                Some("js") => b"text/javascript",
                Some("css") => b"text/css",
                Some("wasm") => b"application/wasm",
                None | Some(_) => b"text/plain",
            };
            match fs::read(path) {
                Ok(content) => Response::from_data(content).with_header(
                    Header::from_bytes(b"Content-Type", content_type).expect("Invalid header"),
                ),
                Err(err) => {
                    println!("{}", err.red());
                    let error_message = format!("😢 Error reading file: {err}");
                    Response::from_string(error_message).with_status_code(500)
                }
            }
        } else {
            let not_found_message = "🚫 404 Not Found!";
            Response::from_string(not_found_message).with_status_code(404)
        };

        let _ = request.respond(response);
    }
}

/// Build a project
fn build(config: &BuildConfig) -> Result<()> {
    println!("🧹 {}", "Cleaning dist".bright_red(),);
    let _ = fs::remove_dir_all(&config.dist);

    println!(
        "🚧 {} (using profile {})",
        "Starting Build".bright_blue(),
        config.profile.redable().cyan()
    );
    std::fs::create_dir_all(&config.dist).context("Creating dist")?;

    let source_wasm_file = build_wasm(config).context("Building wasm")?;
    let (wasm_file, js_file) = wasm_bindgen(config, &source_wasm_file)?;
    if config.profile == BuildProfile::Release {
        optimize_wasm(&wasm_file)?;
    }
    collect_macro_output(config)?;
    generate_html(config, &js_file)?;

    println!(
        "📦 {} {}",
        "Result in".bright_blue(),
        path_str(&config.dist).cyan()
    );

    Ok(())
}

/// Generate the html file to be used
fn generate_html(config: &BuildConfig, js_file: &Path) -> Result<()> {
    let html_file = config.dist.join("index.html");

    let js_file = js_file
        .file_name()
        .ok_or(anyhow!("Js File name not found"))?
        .to_string_lossy();

    let js_reload = match config.profile {
        BuildProfile::Release => "",
        BuildProfile::Dev => {
            r#"
            function check_reload() {
                fetch("./RELOAD")
                    .then(
                        (resp) => {
                            if (resp.ok) {
                                location.reload();
                            }
                            else  {
                                setTimeout(check_reload, 500);
                            }
                        }
                    )
            }
            check_reload();
        "#
        }
    };

    let content = format!(
        r#"
        <!doctype html>
        <html>
            <head>
                <link rel="stylesheet" href="{CSS_OUTPUT_NAME}"/>
            </head>
            <body>
                <div id="{}"></div>
                <script type="module">
                    import init from "./{js_file}";
                    init();
                    {js_reload}
                </script>
            </body>
        </html>
    "#,
        natrix_shared::MOUNT_POINT
    );

    std::fs::write(html_file, content)?;

    Ok(())
}

/// Run wasmbindgen to generate the clue
fn wasm_bindgen(config: &BuildConfig, wasm_file: &PathBuf) -> Result<(PathBuf, PathBuf)> {
    let mut command = process::Command::new("wasm-bindgen");
    command
        .arg(wasm_file)
        .args(["--out-dir", &path_str(&config.dist)])
        .args(["--target", "web"])
        .args(["--out-name", BINDGEN_OUTPUT_NAME])
        .arg("--no-typescript");
    if config.profile == BuildProfile::Dev {
        command.arg("--debug").arg("--keep-debug");
    } else {
        command
            .arg("--remove-name-section")
            .arg("--remove-producers-section");
    }
    run_with_spinner(command, create_spinner("✍️ wasm_bindgen")?)?;

    let js_file = config.dist.join(format!("{BINDGEN_OUTPUT_NAME}.js"));
    if config.profile == BuildProfile::Release {
        minimize_js(&js_file)?;
    }

    Ok((
        config.dist.join(format!("{BINDGEN_OUTPUT_NAME}_bg.wasm")),
        js_file,
    ))
}

/// Minimize the given js file
fn minimize_js(js_file: &PathBuf) -> Result<(), anyhow::Error> {
    let spinner = create_spinner("🗜️ Minimizing JS")?;

    let js_code = fs::read_to_string(js_file)?;
    let allocator = oxc::allocator::Allocator::new();
    let parser = oxc::parser::Parser::new(&allocator, &js_code, oxc::span::SourceType::cjs());

    let mut program = parser.parse().program;
    let minifier = oxc::minifier::Minifier::new(oxc::minifier::MinifierOptions {
        mangle: Some(oxc::minifier::MangleOptions {
            top_level: true,
            ..Default::default()
        }),
        compress: Some(oxc::minifier::CompressOptions::default()),
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
fn build_wasm(config: &BuildConfig) -> Result<PathBuf> {
    let artifact = config.temp_dir.join("cargo");

    let mut command = process::Command::new("cargo");
    command
        .arg("build")
        .args(["--color", "always"])
        .args(["--target", "wasm32-unknown-unknown"])
        .args(["--profile", config.profile.redable()])
        .args([
            "-Z",
            "unstable-options",
            "--artifact-dir",
            &path_str(&artifact),
        ])
        .env(
            natrix_shared::MACRO_OUTPUT_ENV,
            config.temp_dir.join(MACRO_OUTPUT_DIR),
        );
    if config.profile == BuildProfile::Release {
        command
            .args(["-Z", "build-std=core,std,panic_abort"])
            .arg("-Zbuild-std-features=optimize_for_size,panic_immediate_abort")
            .env(
                "RUSTFLAGS",
                "-C target-feature=+bulk-memory -Z fmt-debug=none -Zlocation-detail=none",
            );
    }
    run_with_spinner(command, create_spinner("⚙️ wasm")?).context("Running cargo")?;

    find_wasm(&artifact)
}

/// Alias to Command for consistent code between `process` and `wasm-opt`
#[cfg(not(feature = "bundle-wasm-opt"))]
type Command = std::process::Command;

/// Alias to Command for consistent code between `process` and `wasm-opt`
#[cfg(feature = "bundle-wasm-opt")]
type Command = wasm_opt::integration::Command;

/// Optimize the given wasm file
fn optimize_wasm(wasm_file: &PathBuf) -> Result<(), anyhow::Error> {
    let spinner = create_spinner("🔎 Optimize wasm")?;

    let mut command = Command::new("wasm-opt");
    command
        .arg(wasm_file)
        .arg("-o")
        .arg(wasm_file)
        .arg("--all-features")
        .arg("-tnh")
        .args([
            "-O4",
            "--flatten",
            "--generate-global-effects",
            "--rereloop",
            "-Oz",
            "-Oz",
            "-O3",
            "--monomorphize",
            "-O3",
            "--generate-global-effects",
            "--gufa",
            "--generate-global-effects",
            "--converge",
            "-Oz",
        ]);

    #[cfg(feature = "bundle-wasm-opt")]
    let result = wasm_opt::integration::run_from_command_args(command).is_ok();

    #[cfg(not(feature = "bundle-wasm-opt"))]
    let result = command.status()?.success();

    spinner.finish();
    if !result {
        return Err(anyhow!("Failed to optimize"));
    }
    Ok(())
}

/// Return the path to the first wasm file in the folder
fn find_wasm(directory: &Path) -> Result<PathBuf> {
    for file in directory.read_dir()?.flatten() {
        let path = file.path();
        if let Some(extension) = path.extension() {
            if extension == "wasm" {
                return Ok(file.path());
            }
        }
    }

    Err(anyhow!("Wasm file not found in output directory"))
}

/// Create a spinner with the given msg and finished emoji
fn create_spinner(msg: &str) -> Result<ProgressBar> {
    let spinner = ProgressBar::new_spinner().with_style(
        ProgressStyle::with_template(&format!("{{spinner:.red}} {} {{msg}}", msg.bright_blue()))?
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏-"),
    );
    spinner.enable_steady_tick(Duration::from_millis(100));
    Ok(spinner)
}

/// Convert a path to a `&str` in a lossy way
fn path_str(path: &Path) -> Cow<'_, str> {
    path.as_os_str().to_string_lossy()
}

/// Run the given command displaying the given spinner below it
#[expect(
    clippy::needless_pass_by_value,
    reason = "The spinner isnt usable after this"
)]
fn run_with_spinner(mut command: process::Command, spinner: ProgressBar) -> Result<()> {
    command
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::piped());

    let mut child = command.spawn()?;

    let stderr = child.stderr.take().ok_or(anyhow!("Stderr gone"))?;
    let stderr = BufReader::new(stderr);

    let mut full_output = String::new();
    for line in stderr.lines().map_while(Result::ok) {
        full_output.push_str(&line);
        full_output.push('\n');

        spinner.set_message(line);
    }

    let status = child.wait()?;

    if status.success() {
        spinner.finish_with_message("");
        Ok(())
    } else {
        spinner.finish_with_message("ERROR".red().to_string());
        println!("{full_output}");
        Err(anyhow!("Command exited with non zero status"))
    }
}

/// Collect the outputs of the macros
fn collect_macro_output(config: &BuildConfig) -> Result<()> {
    collect_css(config)?;
    Ok(())
}

/// Collect css from the macro files
fn collect_css(config: &BuildConfig) -> Result<()> {
    let spinner = create_spinner("🎨 Bundling css")?;

    let mut css_content = String::new();
    for file in get_macro_output_files(config).context("Reading macro output")? {
        fs::File::open(file)?.read_to_string(&mut css_content)?;
    }

    if config.profile == BuildProfile::Release {
        css_content = optimize_css(&css_content)?;
    }

    fs::write(config.dist.join(CSS_OUTPUT_NAME), css_content)?;

    spinner.finish();
    Ok(())
}

/// Optimize the given css string
fn optimize_css(css_content: &str) -> Result<String> {
    let mut styles = lightningcss::stylesheet::StyleSheet::parse(
        css_content,
        lightningcss::stylesheet::ParserOptions {
            filename: String::from("<BUNDLED CSS>.css"),
            css_modules: None,
            source_index: 0,
            error_recovery: false,
            warnings: None,
            flags: lightningcss::stylesheet::ParserFlags::empty(),
        },
    )
    .map_err(|err| anyhow!("Failed to parse css {err}"))?;

    let targets = lightningcss::targets::Targets::default();
    styles.minify(lightningcss::stylesheet::MinifyOptions {
        targets,
        unused_symbols: HashSet::new(),
    })?;

    let css_content = styles.to_css(lightningcss::printer::PrinterOptions {
        analyze_dependencies: None,
        minify: true,
        project_root: None,
        pseudo_classes: None,
        targets,
    })?;

    let css_content = css_content.code;

    Ok(css_content)
}

/// Get all files in the sub folders of `MACRO_OUTPUT_DIR`
fn get_macro_output_files(config: &BuildConfig) -> Result<impl Iterator<Item = PathBuf>> {
    Ok(fs::read_dir(config.temp_dir.join(MACRO_OUTPUT_DIR))?
        .flatten()
        .flat_map(|folder| fs::read_dir(folder.path()).into_iter().flatten().flatten())
        .map(|entry| entry.path()))
}
