fn main() {
    println!(
        "cargo:rerun-if-env-changed={}",
        natrix_shared::MACRO_OUTPUT_ENV
    );
}
