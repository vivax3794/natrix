//! Implementation of the `Component` derive macro

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{ItemStruct, parse_quote};

/// A abstract representation of a struct field
pub(crate) struct Field {
    /// The type of the field
    pub(crate) type_: TokenStream,
    /// How one would access the field (identifiers for named structs, a number for tuple)
    pub(crate) access: TokenStream,
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

    let mut reg_dep = quote!();
    let mut dirty_deps_lists = quote!();
    let mut signal_state = quote!();
    let mut pop_state = quote!();
    let mut set_state_calls = quote!();
    let mut set_state_unpack = quote!();

    for field in &fields {
        let type_ = &field.type_;
        let access = &field.access;

        let prefixed = format_ident!("a_{access}");

        where_clause = quote!(#where_clause #type_: ::natrix::macro_ref::State ,);
        reg_dep = quote!(
            #reg_dep
            ::natrix::macro_ref::State::reg_dep(&mut self.#access, key);
        );
        dirty_deps_lists = quote!(
            #dirty_deps_lists
            ::natrix::macro_ref::State::dirty_deps_lists(&mut self.#access, collector);
        );
        signal_state = quote!(
            #signal_state
            <#type_ as ::natrix::macro_ref::State>::SignalState,
        );
        pop_state = quote!(
            #pop_state
            ::natrix::macro_ref::State::pop_state(&mut self.#access),
        );
        set_state_calls = quote!(
            #set_state_calls
            ::natrix::macro_ref::State::set_state(&mut self.#access, #prefixed);
        );
        set_state_unpack = quote!(
            #set_state_unpack
            #prefixed,
        );
    }

    quote! {
        #[automatically_derived]
        impl #impl_generics ::natrix::macro_ref::State for #name #type_generics #where_clause {
            type SignalState = (#signal_state);
            fn reg_dep(&mut self, key: ::natrix::macro_ref::HookKey) {
                #reg_dep
            }
            fn dirty_deps_lists(&mut self, collector: &mut ::std::vec::Vec<::natrix::macro_ref::HookDepListIter>) {
                #dirty_deps_lists
            }
            #[allow(clippy::unused_unit)]
            fn pop_state(&mut self) -> Self::SignalState {
                (#pop_state)
            }
            fn set_state(&mut self, (#set_state_unpack): Self::SignalState) {
                #set_state_calls
            }
        }
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
                access: field.ident.into_token_stream(),
            })
            .collect(),
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .into_iter()
            .enumerate()
            .map(|(index, field)| Field {
                type_: field.ty.to_token_stream(),
                access: proc_macro2::Literal::usize_unsuffixed(index).to_token_stream(),
            })
            .collect(),
    }
}
