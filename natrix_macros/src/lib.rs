//! Derive macros for [Natrix](https://github.com/vivax3794/natrix)
#![warn(missing_docs, clippy::missing_docs_in_private_items)]

extern crate proc_macro;
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
    let (fields, is_named) = get_fields(item.fields);

    let field_count = proc_macro2::Literal::usize_unsuffixed(fields.len());
    let data_name = format_ident!("_{name}Data");
    let signal_state_name = format_ident!("_{name}SignalState");

    let generics = item.generics;
    let mut bounds = generics.clone();
    for type_ in bounds.type_params_mut() {
        type_.bounds.push(parse_quote!('static));
    }

    quote! {
        #[doc(hidden)]
        #(if is_named) {
            struct #data_name #generics {
                #(for field in &fields) {
                    #{field.access.clone()}: ::natrix::macro_ref::Signal<#{field.type_.clone()}, Self>,
                }
            }
            struct #signal_state_name {
                #(for field in &fields) {
                    #{field.access.clone()}: ::natrix::macro_ref::SignalState,
                }
            }
        } #(else) {
            struct #data_name #generics (
                #(for field in &fields) {
                    ::natrix::macro_ref::Signal<#{field.type_.clone()}, Self>,
                }
            );
            struct #signal_state_name (
                #(for _ in &fields) {
                    ::natrix::macro_ref::SignalState,
                }
            );
        }

        #[automatically_derived]
        impl #bounds ::natrix::macro_ref::ComponentData for #data_name #generics {
            type FieldRef<'s> = [&'s mut dyn ::natrix::macro_ref::SignalMethods<Self>; #field_count];
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
        impl #bounds ::natrix::macro_ref::ComponentBase for #name #generics {
            type Data = #data_name #generics;
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
