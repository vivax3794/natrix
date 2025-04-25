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
    println!("cargo::rustc-check-cfg=cfg(nightly)");

    #[cfg(not(feature = "panic_hook"))]
    println!(
        "cargo::warning=`panic_hook` feature disabled, panicking without this feature enabled is instant UB"
    );
}
