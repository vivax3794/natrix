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

use std::fs;
use std::path::PathBuf;

use proc_macro2::TokenStream;
use quote::format_ident;
use syn::{ItemStruct, parse_quote};
use template_quote::{ToTokens, quote};

/// Derive the `ComponentBase` trait for a struct, required for implementing `Component`
///
/// ```rust
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

/// Actual implementation of the macro, split out to make dealing with the different `TokenStream`
/// types easier
fn component_derive_implementation(item: ItemStruct) -> TokenStream {
    let name = item.ident.clone();
    let vis = item.vis;
    let (fields, is_named) = get_fields(item.fields);

    let field_count = proc_macro2::Literal::usize_unsuffixed(fields.len());
    let data_name = format_ident!("_{name}Data");
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

/// Retrive abstract fields from a struct, as well as a boolean indicating wether its a named
/// struct or not (unit structs are considerd named)
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

#[proc_macro]
/// Register global css to be included in the final bundle.
///
/// For most usecases prefer scoped css machinery.
#[expect(
    clippy::missing_panics_doc,
    reason = "Shoudlnt panic in normal build environment"
)]
pub fn global_css(css_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let css = syn::parse_macro_input!(css_input as syn::LitStr);
    let css = css.value();

    #[expect(clippy::expect_used, reason = "This is always set during compilation")]
    let caller_name = std::env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME not set");

    let Ok(output_directory) = std::env::var(natrix_shared::MACRO_OUTPUT_ENV) else {
        return quote!().into();
    };
    let output_directory = PathBuf::from(output_directory);
    let output_directory = output_directory.join(caller_name);

    #[expect(
        clippy::expect_used,
        reason = "This should be valid because the natrix build tool should have made sure of that"
    )]
    std::fs::create_dir_all(&output_directory)
        .expect("Could not create target output directory for crate");

    let name = uuid::Uuid::new_v4().to_string();
    let output_file = output_directory.join(format!("{name}.css"));

    if let Err(err) = fs::write(output_file, css) {
        let err = err.to_string();
        quote!(compiler_error!(#err)).into()
    } else {
        quote!().into()
    }
}
