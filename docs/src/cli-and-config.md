# CLI and Configuration

Natrix comes with a powerful CLI that helps you create, develop, and build your applications. This page explains all the available commands and configuration options.

## The Natrix CLI

The Natrix CLI is a set of tools designed to make working with Natrix projects as smooth as possible. It handles everything from project creation to development servers with hot reloading and optimized production builds.

### Creating New Projects

The `new` command creates a brand new Natrix project with all the necessary files and configuration set up for you.

```bash
natrix new my-awesome-app
```

> [!TIP]
> By default, new projects use nightly Rust for better optimizations and smaller binaries. 
> If you prefer to use stable Rust instead, add the `--stable` flag:
> ```bash
> natrix new my-awesome-app --stable
> ```

This command creates a new project directory with the following structure:

```plain
my-awesome-app/
├── Cargo.toml
├── .gitignore
├── rust-toolchain.toml
├── rustfmt.toml
└── src/
    └── main.rs
```

### Development Server

The `dev` command starts a local development server with live reloading.

```bash
natrix dev
```

The dev server:
- Watches your files for changes
- Automatically rebuilds your app when changes are detected
- Serves your app on a local port (defaults to 8000)
- Sets up WebSocket-based live reloading

> [!TIP]
> You can specify a custom port for the server:
> ```bash
> natrix dev --port 3000
> ```

#### Development Options

```bash
natrix dev [options]
```

- `--port`, `-p`: Set a specific port for the dev server (default: auto-selects starting from 8000)
- `--profile`: Choose the build profile (`dev` or `release`, default: `dev`)
- `--invalidate-cache`: Force asset cache invalidation

### Building for Production

When you're ready to deploy your app, use the `build` command to create an optimized production build.

```bash
natrix build
```

This creates a `dist` folder with everything needed to deploy your application, including:
- Optimized WebAssembly code
- Minified JavaScript
- Bundled and optimized CSS
- HTML entry point
- All assets referenced by your application

#### Build Options

```bash
natrix build [options]
```

- `--dist`, `-d`: Specify the output directory (default: `./dist`)
- `--profile`: Choose the build profile (`dev` or `release`, default: `release`)
- `--invalidate-cache`: Force asset cache invalidation 

## Configuration

Natrix can be configured through your project's `Cargo.toml` file. Add a `[package.metadata.natrix]` section to customize how Natrix builds your application.

### Cache Busting

Control how asset URLs are versioned to ensure browsers load the latest versions:

```toml
[package.metadata.natrix]
cache_bust = "content"  # Options: "none", "content", "timestamp"
```

- `content`: (Default) Creates a hash based on the file content
- `timestamp`: Creates a hash based on the current build time
- `none`: Doesn't add any cache busting

> [!TIP]
> Content-based cache busting is recommended for production as it only changes URLs when the content actually changes, maximizing cache efficiency.

### Base Path

If your app isn't hosted at the root of a domain, you can specify a base path prefix:

```toml
[package.metadata.natrix]
base_path = "/my-app" 
```

This configures all asset URLs to be prefixed with the specified path.

> [!IMPORTANT]
> Always include a leading slash in your `base_path` value.

### Example Configuration

Here's a complete example showing all available configuration options:

```toml
[package.metadata.natrix]
# Use content-based cache busting
cache_bust = "content"
# Deploy to example.com/my-app
base_path = "/my-app"
```

## Build Profiles

Natrix supports two build profiles with different optimization levels:

### Development Profile

The `dev` profile prioritizes build speed and debugging:
- Faster compilation
- Includes debug information
- Minimal optimizations
- Larger output size

### Release Profile

The `release` profile prioritizes performance and size:
- Aggressive code optimization
- CSS and JavaScript minification
- WebAssembly optimizations with `wasm-opt`
- No debug information
- Smaller output size
