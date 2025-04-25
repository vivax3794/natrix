//! Ensure macros rebuild whenever the dependant env vars change
//! Technically speaking this doesnt mean macros have to be *re-built*
//! But that anything using them needs to be.
//! And this is just the easiest way to do that.

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
