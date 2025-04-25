fn main() {
    println!(
        "cargo:rerun-if-env-changed={}",
        natrix_shared::MACRO_OUTPUT_ENV
    );
    println!(
        "cargo:rerun-if-env-changed={}",
        natrix_shared::MACRO_BASE_PATH_ENV
    );
    println!(
        "cargo:rerun-if-env-changed={}",
        natrix_shared::MACRO_INVALIDATE_ENV
    );
}
