//! Sets nightly cfg and emits panic warnings

/// On nightly rust emit `cfg(nightly)`
#[rustversion::nightly]
fn nightly() {
    println!("cargo::rustc-cfg=nightly");
}

/// On nightly rust emit `cfg(nightly)`
#[rustversion::not(nightly)]
fn nightly() {}

fn main() {
    nightly();

    #[cfg(any(debug_assertions, feature = "keep_console_in_release"))]
    println!("cargo::rustc-cfg=console_log");

    println!("cargo::rustc-check-cfg=cfg(nightly)");
    println!("cargo::rustc-check-cfg=cfg(console_log)");
}
