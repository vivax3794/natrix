#[rustversion::nightly]
fn nightly() {
    println!("cargo::rustc-cfg=nightly");
}

#[rustversion::not(nightly)]
fn nightly() {}

fn main() {
    nightly();
    println!("cargo::rustc-check-cfg=cfg(nightly)");
    println!("cargo::rustc-check-cfg=cfg(mutants)");
}
