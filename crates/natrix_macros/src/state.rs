//! Implementation of the `Component` derive macro

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{ItemStruct, parse_quote};

/// A abstract representation of a struct field
pub(crate) struct Field {
    /// The type of the field
    pub(crate) type_: TokenStream,
}

/// Actual implementation of the macro, split out to make dealing with the different `TokenStream`
/// types easier
pub(crate) fn state_derive_implementation(item: ItemStruct) -> TokenStream {
    let name = item.ident.clone();
    let fields = get_fields(item.fields);

    let generics = {
        let mut generics = item.generics;
        for type_ in generics.type_params_mut() {
            type_.bounds.push(parse_quote!('static));
        }
        generics
    };
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let mut where_clause = if let Some(where_clause) = where_clause {
        quote! {#where_clause , }
    } else {
        quote! {where}
    };

    for field in &fields {
        let type_ = &field.type_;
        where_clause = quote!(#where_clause #type_: ::natrix::macro_ref::State ,);
    }

    quote! {
        #[automatically_derived]
        impl #impl_generics ::natrix::macro_ref::State for #name #type_generics #where_clause {}
    }
}

/// Retrieve abstract fields from a struct, as well as a boolean indicating whether its a named
/// struct or not (unit structs are considered named)
pub(crate) fn get_fields(fields: syn::Fields) -> Vec<Field> {
    match fields {
        syn::Fields::Unit => vec![],
        syn::Fields::Named(fields) => fields
            .named
            .into_iter()
            .map(|field| Field {
                type_: field.ty.into_token_stream(),
            })
            .collect(),
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .into_iter()
            .map(|field| Field {
                type_: field.ty.to_token_stream(),
            })
            .collect(),
    }
}
