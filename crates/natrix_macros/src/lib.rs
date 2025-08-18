//! Derive macros for [Natrix](https://github.com/vivax3794/natrix)

extern crate proc_macro;

mod formatting;
mod state;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::{fs, io};

use quote::{format_ident, quote};

/// Create a array of elements based on the format string.
/// The start of the macro is a closure argument list, which should generally be `|ctx: R<Self>|`
/// or similar.
///
/// ```ignore
/// e::div().children(|ctx: R<Self>|, "progress: {}/{}", *ctx.current, *ctx.max)
/// ```
#[proc_macro]
pub fn format_elements(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    formatting::format_elements(input)
}

/// Derive the `State` trait for a struct
#[proc_macro_derive(State)]
pub fn state_derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(item as syn::ItemStruct);
    let result = state::state_derive_implementation(item);
    result.into()
}

/// Convert a struct name to its data variant.
/// This is to allow you to implement methods on `ctx` without having to relay on implementation
/// details
/// ```ignore
/// #[derive(Component)]
/// struct HelloWorld {
///    value: u8,
/// };
///
/// impl natrix::data!(HelloWorld) {
///   fn double(&mut self) {
///     self.value *= 2;
///   }
/// }
/// ```
#[proc_macro]
pub fn data(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let name = syn::parse_macro_input!(input as syn::Ident);
    let name = create_data_struct_name(&name);
    let name = quote! {
        #name
    };
    name.into()
}

/// Create the name for the data struct of a struct
fn create_data_struct_name(name: &syn::Ident) -> syn::Ident {
    format_ident!("_{name}Data")
}

/// If this is the first time a macro is used in this crate we should clear out the target folder
static FIRST_USE_IN_CRATE: AtomicBool = AtomicBool::new(true);

/// Counter to generate unique file names
static FILE_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Emit a file to the target directory
fn emit_file(
    content: natrix_shared::macros::MacroEmisson,
    settings: &natrix_shared::macros::Settings,
) {
    let first_use = FIRST_USE_IN_CRATE.fetch_and(false, Ordering::AcqRel);

    let caller_name =
        std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| String::from("unknown-caller"));

    let output_directory = settings.output_dir.join(caller_name);

    #[expect(
        clippy::expect_used,
        reason = "We should have write permission to target/"
    )]
    {
        if first_use && let Err(err) = std::fs::remove_dir_all(&output_directory) {
            assert!(
                err.kind() == io::ErrorKind::NotFound,
                "Deleting folder failed {err}"
            );
        }
        std::fs::create_dir_all(&output_directory)
            .expect("Could not create target output directory for crate");
    }

    let name = FILE_COUNTER.fetch_add(1, Ordering::AcqRel);
    let output_file = output_directory.join(format!("{name}.natrix"));

    #[expect(
        clippy::expect_used,
        reason = "We dont have any of the types that could cause errors"
    )]
    let encoded = natrix_shared::macros::bincode::encode_to_vec(
        content,
        natrix_shared::macros::bincode_config(),
    )
    .expect("Failed to encode asset information");

    #[expect(
        clippy::expect_used,
        reason = "We should have write permission to target/"
    )]
    fs::write(output_file, encoded).expect("Failed to write output file.");
}

/// Inform the bundling system to include the given asset
/// Will return the url needed to fetch said asset at runtime (including the past path if set).
///
/// ```ignore
/// e::img()
///     .src(asset!("./my_cool_img.png"))
/// ```
#[proc_macro]
#[expect(
    clippy::missing_panics_doc,
    reason = "This can only panic if its not called from cargo, or due to internal macro bugs"
)]
pub fn asset(file_path: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use std::hash::{DefaultHasher, Hash, Hasher};

    let file_path = syn::parse_macro_input!(file_path as syn::LitStr);
    let file_path = file_path.value();

    #[expect(
        clippy::expect_used,
        reason = "This only fails if not called from cargo"
    )]
    let package_directory =
        std::env::var("CARGO_MANIFEST_DIR").expect("Proc macro not called from cargo");
    let package_directory = PathBuf::from(package_directory);
    let file_path = package_directory.join(file_path);

    if !file_path.exists() {
        let err = format!("File {} does not exist.", file_path.display());
        return quote!(compile_error!(#err)).into();
    }

    let Ok(settings) = std::env::var(natrix_shared::MACRO_SETTINGS) else {
        // NOTE:
        // This is not a hard error because running without the bundler is a expected situation
        // (cargo check, ides, etc)
        // But all those situations are also situations where a accurate path is not required as
        // its no runtime (building a natrix application with just `cargo build` is not supported)
        // so we return this path that if it ends up in runtime should hopefully be helpful.
        return quote!("/warn_no_bundler/this_expansion_was_not_via_the_natrix_bundler/as_such_a_proper_path_cant_be_given").into();
    };

    let mut hasher = DefaultHasher::default();

    #[cfg(debug_assertions)]
    file_path.hash(&mut hasher);
    #[cfg(not(debug_assertions))]
    if let Ok(content) = fs::read(&file_path) {
        content.hash(&mut hasher);
    } else {
        file_path.hash(&mut hasher);
    }

    let hash = hasher.finish();
    let hash_base64 = data_encoding::BASE64URL_NOPAD.encode(&hash.to_le_bytes());

    let target = if let Some(file_name) = file_path.file_name() {
        let file_name = file_name.to_string_lossy();
        format!("{hash_base64}-{file_name}")
    } else {
        hash_base64
    };

    #[expect(clippy::expect_used, reason = "We should have a valid base64 string")]
    let settings = data_encoding::BASE64_NOPAD
        .decode(settings.as_bytes())
        .expect("Corrupt base64 in settings var");

    #[expect(clippy::expect_used, reason = "We should have a valid bincode config")]
    let (settings, _): (natrix_shared::macros::Settings, _) =
        natrix_shared::macros::bincode::decode_from_slice(
            &settings,
            natrix_shared::macros::bincode_config(),
        )
        .expect("Failed to decode settings");

    let url = format!("{}/{target}", settings.base_path);

    let result = quote!(#url).into();
    let asset = natrix_shared::macros::MacroEmisson::Asset {
        path: file_path,
        emitted_path: target,
    };

    emit_file(asset, &settings);
    result
}
