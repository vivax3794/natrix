//! Sets nightly cfg and emits panic warnings

/// On nightly rust emit `cfg(nightly)`
#[rustversion::nightly]
fn nightly() {
    println!("cargo::rustc-cfg=nightly");
}

/// On nightly rust emit `cfg(nightly)`
#[rustversion::not(nightly)]
fn nightly() {}

/// Should we log stuff to the console?
#[cfg(any(debug_assertions, feature = "keep_console_in_release"))]
fn log() {
    println!("cargo::rustc-cfg=console_log");
}

/// Should we log stuff to the console?
#[cfg(not(any(debug_assertions, feature = "keep_console_in_release")))]
fn log() {}

fn main() {
    nightly();
    log();

    println!("cargo::rustc-check-cfg=cfg(nightly)");
    println!("cargo::rustc-check-cfg=cfg(console_log)");
    println!("cargo::rustc-check-cfg=cfg({})", natrix_shared::SSG_CFG);
}
