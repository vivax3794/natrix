//! Build system and project generator for natrix
#![forbid(unsafe_code)]
#![deny(
    clippy::todo,
    clippy::unreachable,
    clippy::unwrap_used,
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
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs, process, thread};

use anyhow::{Context, Result, anyhow};
use clap::{Parser, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use lightningcss::visitor::Visit;
use notify::Watcher;
use owo_colors::OwoColorize;
use tiny_http::{Header, Response, Server};

/// The directory to store macro outputs
const MACRO_OUTPUT_DIR: &str = "macro";
/// The name of the js file
const BINDGEN_OUTPUT_NAME: &str = "code";
/// Name of the collected css
const CSS_OUTPUT_NAME: &str = "styles.css";

/// Find the target folder
fn find_target() -> Result<PathBuf> {
    let metadata = cargo_metadata::MetadataCommand::new().no_deps().exec()?;
    let target = metadata.target_directory;
    let target = PathBuf::from(target);
    Ok(target)
}

/// Find the natrix target folder
fn find_target_natrix(mode: BuildProfile) -> Result<PathBuf> {
    let target = find_target()?;
    Ok(target.join(format!("natrix-{}", mode.readable())))
}

/// Get the current target project name
fn get_project_name() -> Result<String> {
    let metadata = cargo_metadata::MetadataCommand::new().no_deps().exec()?;
    let packages = metadata.workspace_default_packages();
    let package = packages.first().ok_or(anyhow!("No package found"))?;

    if packages.len() > 1 {
        return Err(anyhow!(
            "Multiple packages found, please specify the package name"
        ));
    }

    Ok(package.name.clone())
}

/// Natrix CLI
#[derive(Parser)]
enum Cli {
    /// Create a new project
    New {
        /// The name of the project
        name: String,
        /// Use Stable rust
        #[arg(short, long)]
        stable: bool,
    },
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
    /// Location to output build files
    dist: PathBuf,
    /// Location for the temp dir
    temp_dir: PathBuf,
    /// Do live reload
    /// The Some value is the port to use
    live_reload: Option<u16>,
}

impl BuildConfigArgs {
    /// Replace optional arguments (that have defaults) with the defaults for the `dev` subcommand
    fn fill_build_defaults(self) -> Result<BuildConfig> {
        let profile = self.profile.unwrap_or(BuildProfile::Release);
        Ok(BuildConfig {
            profile,
            dist: self.dist.unwrap_or_else(|| PathBuf::from("./dist")),
            temp_dir: find_target_natrix(profile)?,
            live_reload: None,
        })
    }

    /// Replace optional arguments (that have defaults) with the defaults for the `build` subcommand
    fn fill_dev_defaults(self) -> Result<BuildConfig> {
        let profile = self.profile.unwrap_or(BuildProfile::Dev);
        let target = find_target_natrix(profile)?;

        let dist = if let Some(dist) = self.dist {
            dist
        } else {
            target.join("dist")
        };

        let live_reload = if let Ok(port) = get_free_port(9000) {
            Some(port)
        } else {
            println!(
                "{}",
                "No free port found for live reload, disabling it"
                    .red()
                    .bold()
            );
            None
        };

        Ok(BuildConfig {
            profile,
            dist,
            temp_dir: target,
            live_reload,
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
    /// Return a more readable version of this profile name
    fn readable(self) -> &'static str {
        match self {
            Self::Release => "release",
            Self::Dev => "dev",
        }
    }

    /// Return the cargo profile name
    fn cargo(self) -> &'static str {
        match self {
            Self::Release => "release",
            Self::Dev => "dev",
        }
    }

    /// Return the target output folder
    fn target(self) -> &'static str {
        match self {
            Self::Release => "release",
            Self::Dev => "debug",
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::New { name, stable } => generate_project(&name, stable),
        Cli::Dev(config) => do_dev(config),
        Cli::Build(config) => build(&config.fill_build_defaults()?).context("Building application"),
    }
}

/// Generate a new project
fn generate_project(name: &str, stable: bool) -> std::result::Result<(), anyhow::Error> {
    let root = PathBuf::from(name);
    fs::create_dir_all(&root)?;

    let nightly = !stable;

    // we assume the library version is the same as the cli version
    // This means that even if the cli isnt modified it should publish new versions along with the
    // library
    let natrix_version = env!("CARGO_PKG_VERSION");

    let mut natrix_table = format!(r#"version = "{natrix_version}""#);
    if nightly {
        natrix_table = format!(r#"{natrix_table}, features = ["nightly"]"#);
    }
    if let Ok(path) = std::env::var("NATRIX_PATH") {
        natrix_table = format!(r#"{natrix_table}, path = "{path}""#);
    }

    let natrix_decl = format!("natrix = {{ {natrix_table} }}");
    let natrix_decl = natrix_decl.trim();

    let cargo_toml = format!(
        r#"
[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[dependencies]
{natrix_decl}

[profile.release]
opt-level = "z"
codegen-units = 1
lto = "fat"
panic = "abort"
strip = "symbols"
        "#
    );
    fs::write(root.join("Cargo.toml"), cargo_toml)?;

    let gitignore = "
target
dist
    "
    .trim();
    fs::write(root.join(".gitignore"), gitignore)?;

    generate_main_rs(&root, name, nightly)?;

    let rust_fmt = r#"
skip_macro_invocations = ["global_css", "scoped_css"]
    "#
    .trim();
    fs::write(root.join("rustfmt.toml"), rust_fmt)?;

    std::process::Command::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(&root)
        .status()?;

    println!(
        "✨ {} {}",
        "Project created".bright_green(),
        path_str(&root).cyan()
    );
    println!(
        "{}",
        "Run `natrix dev` to start the dev server".bright_blue()
    );

    Ok(())
}

/// Generate the main.rs file for a new project
fn generate_main_rs(root: &Path, name: &str, nightly: bool) -> Result<(), anyhow::Error> {
    let nightly_lints = if nightly {
        "
// This is a nightly only feature to warn you when you are passing certain types across a
// `await` boundary, `natrix` marks multiple types with this attribute to prevent you from
// causing runtime panics.
#![feature(must_not_suspend)]
#![warn(must_not_suspend)]
        "
        .trim()
    } else {
        ""
    };
    let nightly_associated_types_are_optional = if nightly {
        ""
    } else {
        "
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
"
        .trim()
    };
    let main_rs = format!(
        r#"
{nightly_lints}
// Panicking in a wasm module will cause the state to be invalid
// And it might cause UB on the next event handler execution.
// (By default natrix uses a panic hook that blocks further event handler calls after a panic)
#![deny(clippy::panic, clippy::unwrap_used, clippy::expect_used)]
// These are more strict anti panic lints that you might want to enable
// #![warn(clippy::arithmetic_side_effects, clippy::indexing_slicing, clippy::unreachable)]

use natrix::prelude::*;

mod css {{
    use natrix::prelude::scoped_css;
    scoped_css!("
        .hello_world {{
            font-size: 6rem;
            color: red;
        }}
    ");
}}

#[derive(Component)]
struct HelloWorld;

impl Component for HelloWorld {{
    {nightly_associated_types_are_optional}
    fn render() -> impl Element<Self> {{
        e::h1().text("Hello {name}").class(css::HELLO_WORLD)
    }}
}}

fn main() {{
    mount(HelloWorld);
}}
"#
    );
    let src = root.join("src");
    fs::create_dir_all(&src)?;
    fs::write(src.join("main.rs"), main_rs)?;

    let channel = if nightly { "nightly" } else { "stable" };

    let toolchain_toml = format!(
        r#"
        [toolchain]
        channel = "{channel}"
        targets = ["wasm32-unknown-unknown"]
        "#
    );
    fs::write(root.join("rust-toolchain.toml"), toolchain_toml)?;

    Ok(())
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
    thread::spawn(|| spawn_server(dist));

    if let Some(port) = config.live_reload {
        thread::spawn(move || spawn_websocket(port, rx_reload));
    }

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

/// Spawn a websocket server to send reload signals
#[expect(
    clippy::expect_used,
    clippy::needless_pass_by_value,
    reason = "This is running in a thread"
)]
fn spawn_websocket(port: u16, reload_signal: Receiver<()>) {
    let server = TcpListener::bind(("127.0.1", port)).expect("Failed to bind websocket");
    let clients = Arc::new(Mutex::new(Vec::new()));

    let clients_2 = clients.clone();
    thread::spawn(move || {
        for stream in server.incoming() {
            let stream = stream.expect("Failed to accept connection");
            let ws = tungstenite::accept(stream).expect("Failed to accept websocket");
            let mut clients = clients_2.lock().expect("Mutex gone");
            clients.push(ws);
        }
    });

    loop {
        if let Ok(()) = reload_signal.recv() {
            let mut clients = clients.lock().expect("Mutex gone");
            for mut client in clients.drain(..) {
                let _ = client.write(tungstenite::Message::from("RELOAD NOW PLS"));
                client.flush().expect("Failed to flush");
            }
        }
    }
}

/// Find a free port
fn get_free_port(mut preferred: u16) -> Result<u16, &'static str> {
    loop {
        if TcpListener::bind(("127.0.0.1", preferred)).is_ok() {
            return Ok(preferred);
        }
        if let Some(new_port) = preferred.checked_add(1) {
            preferred = new_port;
        } else {
            return Err("No free port found");
        }
    }
}

/// Spawn a dev server for serving files
#[expect(
    clippy::expect_used,
    clippy::needless_pass_by_value,
    reason = "This is running in a thread"
)]
fn spawn_server(folder: PathBuf) {
    let server = Server::http((
        "127.0.0.1",
        get_free_port(8000).expect("Failed to find free port for server"),
    ))
    .expect("Failed to start server");
    let port = server
        .server_addr()
        .to_ip()
        .expect("Failed to get ip")
        .port();
    println!(
        "{}{}",
        "🚀 Dev server running at http://localhost:".green(),
        port.to_string().bright_red()
    );

    for request in server.incoming_requests() {
        let url = request.url();
        let path = if url == "/" {
            folder.join("index.html")
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
    println!("🧹 {}", "Cleaning dist".bright_black(),);
    let _ = fs::remove_dir_all(&config.dist);

    println!(
        "🚧 {} (using profile {})",
        "Starting Build".bright_blue(),
        config.profile.readable().cyan()
    );
    std::fs::create_dir_all(&config.dist).context("Creating dist")?;

    if !is_feature_enabled("panic_hook", true)? {
        println!(
            "{}",
            "⚠️ `panic_hook` feature is disabled, panicking without this feature enabled is instant UB"
                .red()
                .bold()
        );
    }

    let source_wasm_file = build_wasm(config).context("Building wasm")?;
    let (wasm_file, js_file) = wasm_bindgen(config, &source_wasm_file)?;
    if config.profile == BuildProfile::Release {
        optimize_wasm(&wasm_file)?;
    }
    collect_macro_output(config, &wasm_file)?;
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

    std::fs::write(html_file, content.trim())?;

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
            .arg("--remove-producers-section")
            .arg("--no-demangle");
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
        compress: Some(oxc::minifier::CompressOptions {
            drop_console: true,
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
fn build_wasm(config: &BuildConfig) -> Result<PathBuf> {
    let rustc_version_meta = rustc_version::version_meta()?;
    let rustc_is_nightly = rustc_version_meta.channel == rustc_version::Channel::Nightly;

    let mut command = process::Command::new("cargo");
    command
        .arg("build")
        .args(["--color", "always"])
        .args(["--target", "wasm32-unknown-unknown"])
        .args(["--profile", config.profile.cargo()])
        .env(
            natrix_shared::MACRO_OUTPUT_ENV,
            config.temp_dir.join(MACRO_OUTPUT_DIR),
        );
    if config.profile == BuildProfile::Release {
        let mut rustc_flags =
            String::from("-C target-feature=+bulk-memory -C target-feature=+reference-types ");
        if rustc_is_nightly {
            let mut std_features = String::from("optimize_for_size");
            if !is_feature_enabled("panic_hook", true)? {
                std_features.push_str(",panic_immediate_abort");
            }

            command
                .args(["-Z", "build-std=core,std,panic_abort"])
                .arg(format!("-Zbuild-std-features={std_features}"));
            rustc_flags.push_str("-Zfmt-debug=none -Zlocation-detail=none");
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
    run_with_spinner(command, create_spinner("⚙️ wasm")?).context("Running cargo")?;

    find_wasm(config).context("Finding wasm file")
}

/// Find if the specified feature is enabled for natrix
fn is_feature_enabled(feature: &str, is_default: bool) -> Result<bool> {
    let metadata = cargo_metadata::MetadataCommand::new().no_deps().exec()?;
    let packages = metadata.workspace_default_packages();
    let package = packages.first().ok_or(anyhow!("No package found"))?;
    let natrix = package.dependencies.iter().find(|x| x.name == "natrix");

    Ok(if let Some(natrix) = natrix {
        if natrix.features.iter().any(|feat| feat == feature) {
            true
        } else {
            is_default && natrix.uses_default_features
        }
    } else {
        println!("{}", "⚠️ Natrix not found in dependencies".yellow().bold());
        false
    })
}

/// Return the path to the first wasm file in the folder
fn find_wasm(config: &BuildConfig) -> Result<PathBuf> {
    let target = find_target()?;
    let name = get_project_name()?;
    let target = target
        .join("wasm32-unknown-unknown")
        .join(config.profile.target());

    if let Some(wasm) = search_dir_for_wasm(&target, &name)? {
        return Ok(wasm);
    }

    if let Some(wasm) = search_dir_for_wasm(&target.join("deps"), &name)? {
        return Ok(wasm);
    }

    Err(anyhow!("Wasm file not found in {}", path_str(&target)))
}

/// Search the given directory for a wasm file
fn search_dir_for_wasm(target: &Path, name: &str) -> Result<Option<PathBuf>, anyhow::Error> {
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
fn optimize_wasm(wasm_file: &PathBuf) -> Result<(), anyhow::Error> {
    let spinner = create_spinner("🔎 Optimize wasm")?;

    let mut command = process::Command::new("wasm-opt");
    command
        .arg(wasm_file)
        .arg("-o")
        .arg(wasm_file)
        .arg("--enable-bulk-memory")
        .arg("--enable-reference-types")
        .arg("--strip-producers");
    if !is_feature_enabled("panic_hook", true)? {
        command.arg("--traps-never-happen");
    }
    command.args([
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

    let result = command.status()?.success();

    spinner.finish();
    if !result {
        return Err(anyhow!("Failed to optimize"));
    }
    Ok(())
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
fn collect_macro_output(config: &BuildConfig, wasm_file: &Path) -> Result<()> {
    collect_css(config, wasm_file)?;
    Ok(())
}

/// Collect css from the macro files
fn collect_css(config: &BuildConfig, wasm_file: &Path) -> Result<()> {
    let spinner = create_spinner("🎨 Bundling css")?;

    let mut css_content = String::new();
    for file in get_macro_output_files(config).context("Reading macro output")? {
        fs::File::open(file)?.read_to_string(&mut css_content)?;
    }

    if config.profile == BuildProfile::Release {
        css_content = optimize_css(&css_content, wasm_file)?;
    }

    fs::write(config.dist.join(CSS_OUTPUT_NAME), css_content)?;

    spinner.finish();
    Ok(())
}

/// Optimize the given css string
fn optimize_css(css_content: &str, wasm_file: &Path) -> Result<String> {
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

    let wasm_strings = get_wasm_strings(wasm_file)?;
    let mut unused_symbols = get_symbols(&mut styles);
    // `wasm_strings` is a vec of data sections, so we need to check if the symbol is in any of
    // them as wasm optimizes multiple string literals to the same section
    unused_symbols.retain(|symbol| wasm_strings.iter().all(|x| !x.contains(symbol)));

    let targets = lightningcss::targets::Targets::default();
    styles.minify(lightningcss::stylesheet::MinifyOptions {
        targets,
        unused_symbols,
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

/// Visitor to extract symbosl from a stylesheet
struct SymbolVisitor {
    /// The collected symbols
    symbols: HashSet<String>,
    /// Symbols the should always be kept
    keep: HashSet<String>,
}

impl<'i> lightningcss::visitor::Visitor<'i> for SymbolVisitor {
    type Error = std::convert::Infallible;
    fn visit_types(&self) -> lightningcss::visitor::VisitTypes {
        lightningcss::visit_types!(SELECTORS | RULES)
    }

    fn visit_rule(
        &mut self,
        rule: &mut lightningcss::rules::CssRule<'i>,
    ) -> std::result::Result<(), Self::Error> {
        if let lightningcss::rules::CssRule::Unknown(unknown_rule) = rule {
            if unknown_rule.name == "keep" {
                let tokens = &unknown_rule.prelude.0;
                if let Some(token) = tokens.first() {
                    match token {
                        lightningcss::properties::custom::TokenOrValue::Token(
                            lightningcss::properties::custom::Token::Ident(ident),
                        ) => {
                            let ident = ident.to_string();
                            self.keep.insert(ident);
                        }
                        lightningcss::properties::custom::TokenOrValue::DashedIdent(ident) => {
                            let ident = ident.to_string();
                            self.keep.insert(ident);
                        }
                        _ => (),
                    }
                }
                *rule = lightningcss::rules::CssRule::Ignored;
            }
        }
        rule.visit_children(self)
    }

    fn visit_selector(
        &mut self,
        selector: &mut lightningcss::selector::Selector<'i>,
    ) -> std::result::Result<(), Self::Error> {
        use lightningcss::selector::Component;
        for part in selector.iter_mut_raw_match_order() {
            match part {
                Component::Class(class) => {
                    self.symbols.insert(class.to_string());
                }
                Component::ID(id) => {
                    self.symbols.insert(id.to_string());
                }
                Component::Negation(lst) | Component::Is(lst) | Component::Where(lst) => {
                    for selector in lst.iter_mut() {
                        self.visit_selector(selector)?;
                    }
                }
                _ => (),
            }
        }
        Ok(())
    }

    fn visit_selector_list(
        &mut self,
        selectors: &mut lightningcss::selector::SelectorList<'i>,
    ) -> std::result::Result<(), Self::Error> {
        for selector in &mut selectors.0 {
            self.visit_selector(selector)?;
        }
        Ok(())
    }
}

/// Get the symbols to DCE in a style sheet
fn get_symbols(stylesheet: &mut lightningcss::stylesheet::StyleSheet) -> HashSet<String> {
    let mut visitor = SymbolVisitor {
        symbols: HashSet::new(),
        keep: HashSet::new(),
    };
    let _ = stylesheet.visit(&mut visitor);
    visitor.symbols.difference(&visitor.keep).cloned().collect()
}

/// Get all files in the sub folders of `MACRO_OUTPUT_DIR`
fn get_macro_output_files(config: &BuildConfig) -> Result<impl Iterator<Item = PathBuf>> {
    Ok(fs::read_dir(config.temp_dir.join(MACRO_OUTPUT_DIR))?
        .flatten()
        .flat_map(|folder| fs::read_dir(folder.path()).into_iter().flatten().flatten())
        .map(|entry| entry.path()))
}

/// Get the strings from a wasm file
fn get_wasm_strings(wasm_file: &Path) -> Result<Vec<String>> {
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
