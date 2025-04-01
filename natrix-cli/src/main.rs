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
use std::io::{BufRead, BufReader};
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

/// Retrive the project name to use
fn get_project_name() -> Result<String> {
    let current_dir = std::env::current_dir()?;
    if let Some(name) = current_dir.file_name() {
        Ok(name.to_string_lossy().into_owned())
    } else {
        Ok(String::from("Root"))
    }
}

/// Get the temporary directory for this
fn temp_dir() -> Result<PathBuf> {
    let project = get_project_name()?;
    let temp = std::env::temp_dir();
    let project_temp = temp.join("natrix").join(project);

    std::fs::create_dir_all(&project_temp)?;

    Ok(project_temp)
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
    /// Location to putput build files
    #[arg(short, long)]
    dist: Option<PathBuf>,
}

/// Settings for building the server
struct BuildConfig {
    /// Build profile to use
    profile: BuildProfile,
    /// Location to putput build files
    dist: PathBuf,
}

impl BuildConfigArgs {
    /// Replace optional arguments (that have defaults) with the defaults for the `dev` subcommand
    fn fill_build_defaults(self) -> BuildConfig {
        BuildConfig {
            profile: self.profile.unwrap_or(BuildProfile::Release),
            dist: self.dist.unwrap_or_else(|| PathBuf::from("./dist")),
        }
    }

    /// Replace optional arguments (that have defaults) with the defaults for the `build` subcommand
    fn fill_dev_defaults(self) -> Result<BuildConfig> {
        let dist = if let Some(dist) = self.dist {
            dist
        } else {
            temp_dir()?.join("__dist")
        };

        Ok(BuildConfig {
            profile: self.profile.unwrap_or(BuildProfile::Dev),
            dist,
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
        Cli::Build(config) => build(&config.fill_build_defaults()).context("Building application"),
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
    println!("🌐 {}", "Serving page on http://localhost:8000".green());

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

        let response = if path.exists() && path.is_file() {
            let content_type: &[u8] = match path.extension().and_then(|x| x.to_str()) {
                Some("html") => b"text/html",
                Some("js") => b"text/javascript",
                Some("css") => b"text/css",
                Some("wasm") => b"application/wasm",
                None | Some(_) => b"text/plain",
            };
            println!(
                "Serving {} ({})",
                &path_str(&path).green(),
                String::from_utf8(content_type.to_vec())
                    .unwrap_or_default()
                    .bright_green()
            );
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
            println!("NOT FOUND {}", path_str(&path).red());
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
        .args(["--out-name", "output"])
        .arg("--no-typescript")
        .arg("--remove-name-section")
        .arg("--remove-producers-section");
    run_with_spinner(command, create_spinner("✍️ wasm_bindgen")?)?;

    let js_file = config.dist.join("output.js");
    if config.profile == BuildProfile::Release {
        let spinner = create_spinner("🗜️ Minimizing JS")?;
        let js_code = fs::read_to_string(&js_file)?;

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

        std::fs::write(&js_file, js_code)?;
        spinner.finish();
    }

    Ok((config.dist.join("output_bg.wasm"), js_file))
}

/// Build the project wasm
fn build_wasm(config: &BuildConfig) -> Result<PathBuf> {
    let artifact = temp_dir()?.join("__cargo");

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
        ]);
    run_with_spinner(command, create_spinner("⚙️ wasm")?).context("Running cargo")?;

    find_wasm(&artifact)
}

/// Optimize the given wasm file
fn optimize_wasm(wasm_file: &PathBuf) -> Result<(), anyhow::Error> {
    let spinner = create_spinner("🔎 Optimize wasm")?;
    wasm_opt::OptimizationOptions::new_opt_level_3()
        .shrink_level(wasm_opt::ShrinkLevel::Level2)
        .set_converge()
        .add_pass(wasm_opt::Pass::Monomorphize)
        .add_pass(wasm_opt::Pass::GenerateGlobalEffects)
        .add_pass(wasm_opt::Pass::Gufa)
        .run(wasm_file, wasm_file)?;
    spinner.finish();
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
