//! Derive macros for [Natrix](https://github.com/vivax3794/natrix)
#![forbid(
    unsafe_code,
    clippy::todo,
    clippy::unreachable,
    clippy::unwrap_used,
    clippy::unreachable,
    clippy::indexing_slicing
)]
#![deny(
    clippy::dbg_macro,
    clippy::expect_used,
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason,
    clippy::arithmetic_side_effects
)]
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::pedantic,
    unfulfilled_lint_expectations
)]

extern crate proc_macro;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::{fs, io};

use proc_macro2::TokenStream;
use quote::format_ident;
use syn::{ItemStruct, parse_quote};
use template_quote::{ToTokens, quote};

/// Derive the `ComponentBase` trait for a struct, required for implementing `Component`
///
/// ```ignore
/// #[derive(Component)]
/// struct HelloWorld;
///
/// impl Component for HelloWorld {
///     fn render() -> impl Element<Self::Data> {
///         e::h1().text("Hello World")
///     }
/// }
/// ```
#[proc_macro_derive(Component)]
pub fn component_derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(item as ItemStruct);
    let result = component_derive_implementation(item);
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

/// Actual implementation of the macro, split out to make dealing with the different `TokenStream`
/// types easier
fn component_derive_implementation(item: ItemStruct) -> TokenStream {
    let name = item.ident.clone();
    let vis = item.vis;
    let (fields, is_named) = get_fields(item.fields);

    let field_count = proc_macro2::Literal::usize_unsuffixed(fields.len());
    let data_name = create_data_struct_name(&name);
    let signal_state_name = format_ident!("_{name}SignalState");

    let mut generics = item.generics;
    for type_ in generics.type_params_mut() {
        type_.bounds.push(parse_quote!('static));
    }
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[doc(hidden)]
        #(if is_named) {
            #vis struct #data_name #generics {
                #(for field in &fields) {
                    #{field.access.clone()}: ::natrix::macro_ref::Signal<#{field.type_.clone()}>,
                }
            }
            #vis struct #signal_state_name {
                #(for field in &fields) {
                    #{field.access.clone()}: ::natrix::macro_ref::SignalState,
                }
            }
        } #(else) {
            #vis struct #data_name #generics (
                #(for field in &fields) {
                    ::natrix::macro_ref::Signal<#{field.type_.clone()}>,
                }
            );
            #vis struct #signal_state_name (
                #(for _ in &fields) {
                    ::natrix::macro_ref::SignalState,
                }
            );
        }

        #[automatically_derived]
        impl #impl_generics ::natrix::macro_ref::ComponentData for #data_name #type_generics #where_clause {
            type FieldRef<'s> = [&'s mut dyn ::natrix::macro_ref::SignalMethods; #field_count];
            type SignalState = #signal_state_name;

            fn signals_mut(&mut self) -> Self::FieldRef<'_> {
                [
                    #(for field in &fields) {
                        &mut self.#{field.access.clone()},
                    }
                ]
            }

            fn pop_signals(&mut self) -> Self::SignalState {
                #(if is_named) {
                    #signal_state_name {
                        #(for field in &fields) {
                            #{field.access.clone()}: self.#{field.access.clone()}.pop_state(),
                        }
                    }
                } #(else) {
                    #signal_state_name (
                        #(for field in &fields) {
                            self.#{field.access.clone()}.pop_state(),
                        }
                    )
                }
            }

            fn set_signals(&mut self, state: Self::SignalState) {
                #(for field in &fields) {
                    self.#{field.access.clone()}.set_state(state.#{field.access.clone()});
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics ::natrix::macro_ref::ComponentBase for #name #type_generics #where_clause {
            type Data = #data_name #type_generics;
             fn into_data(self) -> Self::Data {
                #(if is_named) {
                    #data_name {
                        #(for field in fields) {
                            #{field.access.clone()}: ::natrix::macro_ref::Signal::new(self.#{field.access}),
                        }
                    }
                } #(else) {
                    #data_name(
                        #(for field in fields) {
                            ::natrix::macro_ref::Signal::new(self.#{field.access}),
                        }
                    )
                }
            }
        }
    }
}

/// Retrieve abstract fields from a struct, as well as a boolean indicating whether its a named
/// struct or not (unit structs are considered named)
fn get_fields(fields: syn::Fields) -> (Vec<Field>, bool) {
    match fields {
        syn::Fields::Unit => (vec![], true),
        syn::Fields::Named(fields) => (
            fields
                .named
                .into_iter()
                .map(|field| Field {
                    type_: field.ty.into_token_stream(),
                    access: field.ident.into_token_stream(),
                })
                .collect(),
            true,
        ),
        syn::Fields::Unnamed(fields) => (
            fields
                .unnamed
                .into_iter()
                .enumerate()
                .map(|(index, field)| Field {
                    type_: field.ty.to_token_stream(),
                    access: proc_macro2::Literal::usize_unsuffixed(index).to_token_stream(),
                })
                .collect(),
            false,
        ),
    }
}

/// A abstract representation of a struct field
struct Field {
    /// The type of the field
    type_: TokenStream,
    /// How one would access the field (identifiers for named structs, a number for tuple)
    access: TokenStream,
}

/// If this is the first time a macro is used in this crate we should clear out the target folder
static FIRST_USE_IN_CRATE: AtomicBool = AtomicBool::new(true);

/// Counter to generate unique file names
static FILE_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Register global css to be included in the final bundle.
///
/// For most usecases prefer scoped css machinery.
#[proc_macro]
pub fn global_css(css_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let css = syn::parse_macro_input!(css_input as syn::LitStr);
    let css = css.value();

    emit_file(css, "css").into()
}

/// Emit the css to the target directory
fn emit_file(content: impl AsRef<[u8]>, extension: &str) -> TokenStream {
    let first_use = FIRST_USE_IN_CRATE.fetch_and(false, Ordering::AcqRel);

    let caller_name =
        std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| String::from("unknown-caller"));

    let Ok(output_directory) = std::env::var(natrix_shared::MACRO_OUTPUT_ENV) else {
        return quote!();
    };
    let output_directory = PathBuf::from(output_directory);
    let output_directory = output_directory.join(caller_name);

    #[expect(
        clippy::expect_used,
        reason = "This should be valid because the natrix build tool should have made sure of that"
    )]
    {
        if first_use {
            if let Err(err) = std::fs::remove_dir_all(&output_directory) {
                assert!(
                    err.kind() == io::ErrorKind::NotFound,
                    "Deleting folder failed {err}"
                );
            }
        }
        std::fs::create_dir_all(&output_directory)
            .expect("Could not create target output directory for crate");
    }

    let name = FILE_COUNTER.fetch_add(1, Ordering::AcqRel);
    let output_file = output_directory.join(format!("{name}.{extension}"));

    if let Err(err) = fs::write(output_file, content) {
        let err = err.to_string();
        quote!(compile_error!(#err))
    } else {
        quote!()
    }
}

/// Create scoped css for a component.
///
/// This generates a set of constants for every class and id in the css.
///
/// ```rust
/// # use natrix_macros::scoped_css;
/// scoped_css!("
///    .hello {
///        color: red;
///     }
///    button .test {
///        color: blue;
///    }
/// ");
/// ```
/// Will expand to (actual string values will be random):
/// ```rust
/// pub(crate) const HELLO: &str = "hello-123456";
/// pub(crate) const TEST: &str = "test-123456";
/// ```
/// (`pub(crate)` is always used as the visibility)
///
/// While emitting something like this to the css bundle:
/// ```css
/// .hello-123456 {
///   color: red;   
/// }
/// button .test-123456 {
///  color: blue;
/// }
/// ```
///
/// Its is generally recommended to use this macro in a module to make it clear where constants are
/// coming from
/// ```ignore
/// mod css {
///     scoped_css!("
///     .hello {
///         color: red;
///     }
///     ");
/// }
///
/// // ...
/// e::div().class(css::HELLO);
/// ```
///
/// # Consistency
/// The generated string literals are not guaranteed to be the same between builds.
/// Their exact format is not covered by the public API and may change in the future.
#[proc_macro]
#[expect(
    clippy::missing_panics_doc,
    reason = "This can only panic if its not called from cargo"
)]
#[cfg(feature = "scoped_css")]
pub fn scoped_css(css_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use convert_case::{Case, Casing};

    let css = syn::parse_macro_input!(css_input as syn::LitStr);
    let css = css.value();

    let caller_name =
        std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| String::from("unknown-caller"));

    #[expect(clippy::expect_used, reason = "Pattern should be valid")]
    let styles = lightningcss::stylesheet::StyleSheet::parse(
        &css,
        lightningcss::stylesheet::ParserOptions {
            filename: caller_name,
            css_modules: Some(lightningcss::css_modules::Config {
                dashed_idents: true,
                container: true,
                custom_idents: true,
                animation: true,
                grid: true,
                pure: true,
                pattern: lightningcss::css_modules::Pattern::parse("[content-hash]-[local]")
                    .expect("Failed to parse pattern"),
            }),
            source_index: 0,
            error_recovery: false,
            warnings: None,
            flags: lightningcss::stylesheet::ParserFlags::empty(),
        },
    );
    let styles = match styles {
        Ok(styles) => styles,
        Err(err) => {
            let err = err.to_string();
            return quote!(compile_error!(#err)).into();
        }
    };

    #[expect(
        clippy::expect_used,
        reason = "If the css can be parsed it should be valid to serialize it"
    )]
    let css_result = styles
        .to_css(lightningcss::stylesheet::PrinterOptions {
            minify: false,
            project_root: None,
            analyze_dependencies: None,
            pseudo_classes: None,
            targets: lightningcss::targets::Targets::default(),
        })
        .expect("Failed to convert css to string");

    #[expect(
        clippy::expect_used,
        reason = "We set the css_modules value to true, so this field should be present"
    )]
    let expand = css_result.exports.expect("Exports not found");
    let mut consts = Vec::with_capacity(expand.len());
    for (name, export) in expand {
        let new_name = export.name;
        let const_name = name.to_case(Case::Constant);
        let const_name = format_ident!("{const_name}");

        consts.push(quote! {
            #[doc = #name]
            pub(crate) const #const_name: &str = #new_name;
        });
    }

    let emit_css_result = emit_file(css_result.code, "css");
    quote! {
        #(for const_ in consts) {
            #const_
        }
        #emit_css_result
    }
    .into()
}

/// Generate a ad-hoc class with the specific style
/// These names will be identical for indetical styling.
/// This is a natrixses answer to tailwindcss, we do not do short hand classes
/// But instead generate a class for every unique style
/// This still isnt as strong as tailwindcss in terms of modifiers (`:hover`, `:active`, etc)
///
/// If a element requires many of these classes, consider using a scoped css macro instead of
/// generate one common class for all properties
#[proc_macro]
#[cfg(feature = "inline_css")]
pub fn style(css: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let css = syn::parse_macro_input!(css as syn::LitStr);
    let css = css.value();

    let mut hasher = DefaultHasher::default();
    css.hash(&mut hasher);
    let hash = hasher.finish();
    let hash = data_encoding::BASE64URL_NOPAD.encode(&hash.to_le_bytes());

    let class_name = format!("inline-{hash}");

    let css = format!(".{class_name} {{ {css} }}");
    emit_file(css, "css");

    quote!(#class_name).into()
}

/// Inform the bundling system to include the given asset
/// Will return the url needed to fetch said asset at runtime.
///
/// ```ignore
/// e::img()
///     .src(asset!("./my_cool_img.png"))
/// ```
#[cfg(feature = "assets")]
#[proc_macro]
#[expect(
    clippy::missing_panics_doc,
    reason = "This can only panic if its not called from cargo"
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

    let base_path = std::env::var(natrix_shared::MACRO_BASE_PATH_ENV).unwrap_or_default();
    let url = format!("{base_path}/{target}");

    let result = quote!(#url).into();
    let asset = natrix_shared::Asset {
        path: file_path,
        emitted_path: target,
    };

    #[expect(
        clippy::expect_used,
        reason = "We dont have any of the types that could cause errors"
    )]
    let asset_encoded =
        natrix_shared::bincode::encode_to_vec(asset, natrix_shared::bincode_config())
            .expect("Failed to encode asset information");

    emit_file(asset_encoded, "asset");
    result
}
