//! Sets nightly cfg

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
    println!("cargo::rustc-check-cfg=cfg(nightly)");
}
