//! Implementation of the `Component` derive macro

// TODO: Make components impl `Element`

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{ItemStruct, parse_quote};

use super::create_data_struct_name;

/// A abstract representation of a struct field
pub(crate) struct Field {
    /// The type of the field
    pub(crate) type_: TokenStream,
    /// How one would access the field (identifiers for named structs, a number for tuple)
    pub(crate) access: TokenStream,
}

/// Container for generic-related parameters
pub(crate) struct GenericParams<'a> {
    /// Generic parameters for the impl block
    pub(crate) impl_generics: &'a syn::ImplGenerics<'a>,
    /// Generic parameters for the type
    pub(crate) type_generics: &'a syn::TypeGenerics<'a>,
    /// The where clause for the impl block
    pub(crate) where_clause: Option<&'a syn::WhereClause>,
}

/// Container for component implementation parameters
pub(crate) struct ComponentImplParams<'a> {
    /// The name of the data struct
    pub(crate) data_name: &'a syn::Ident,
    /// The name of the signal state struct
    pub(crate) signal_state_name: &'a syn::Ident,
    /// The number of fields in the struct
    pub(crate) field_count: &'a proc_macro2::Literal,
    /// The fields of the struct
    pub(crate) fields: &'a [Field],
    /// Whether the struct is named or not
    pub(crate) is_named: bool,
}

/// Actual implementation of the macro, split out to make dealing with the different `TokenStream`
/// types easier
pub(crate) fn component_derive_implementation(item: ItemStruct) -> TokenStream {
    let name = item.ident.clone();
    let vis = item.vis;
    let (fields, is_named) = get_fields(item.fields);
    let field_count = proc_macro2::Literal::usize_unsuffixed(fields.len());
    let data_name = create_data_struct_name(&name);
    let signal_state_name = format_ident!("_{name}SignalState");

    let generics = {
        let mut generics = item.generics;
        for type_ in generics.type_params_mut() {
            type_.bounds.push(parse_quote!('static));
        }
        generics
    };
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let generic_params = GenericParams {
        impl_generics: &impl_generics,
        type_generics: &type_generics,
        where_clause,
    };

    let component_params = ComponentImplParams {
        data_name: &data_name,
        signal_state_name: &signal_state_name,
        field_count: &field_count,
        fields: &fields,
        is_named,
    };

    let mut tokens = TokenStream::new();

    // Generate struct definitions
    tokens.extend({
        let vis: &syn::Visibility = &vis;
        let data_name: &syn::Ident = &data_name;
        let signal_state_name: &syn::Ident = &signal_state_name;
        let generics: &syn::Generics = &generics;
        let fields: &[Field] = &fields;
        if is_named {
            generate_named_struct_definitions(vis, data_name, signal_state_name, generics, fields)
        } else {
            generate_tuple_struct_definitions(vis, data_name, signal_state_name, generics, fields)
        }
    });

    // Generate ComponentData implementation
    tokens.extend(generate_component_data_impl(
        &generic_params,
        &component_params,
    ));

    // Generate ComponentBase implementation
    tokens.extend(generate_component_base_impl(
        &generic_params,
        &name,
        &component_params,
    ));

    tokens
}

/// Generate struct definitions for named structs
pub(crate) fn generate_named_struct_definitions(
    vis: &syn::Visibility,
    data_name: &syn::Ident,
    signal_state_name: &syn::Ident,
    generics: &syn::Generics,
    fields: &[Field],
) -> TokenStream {
    let mut data_struct_fields = TokenStream::new();
    let mut signal_state_fields = TokenStream::new();

    for field in fields {
        let access = &field.access;
        let type_ = &field.type_;
        data_struct_fields.extend(quote! { #access: ::natrix::macro_ref::Signal<#type_>, });
        signal_state_fields.extend(quote! { #access: ::natrix::macro_ref::SignalState, });
    }

    quote! {
        #[doc(hidden)]
        #vis struct #data_name #generics {
            #data_struct_fields
        }

        #vis struct #signal_state_name {
            #signal_state_fields
        }
    }
}

/// Generate struct definitions for tuple structs
pub(crate) fn generate_tuple_struct_definitions(
    vis: &syn::Visibility,
    data_name: &syn::Ident,
    signal_state_name: &syn::Ident,
    generics: &syn::Generics,
    fields: &[Field],
) -> TokenStream {
    let mut data_struct_fields = TokenStream::new();
    let mut signal_state_fields = TokenStream::new();

    for field in fields {
        let type_ = &field.type_;
        data_struct_fields.extend(quote! { ::natrix::macro_ref::Signal<#type_>, });
    }

    for _ in fields {
        signal_state_fields.extend(quote! { ::natrix::macro_ref::SignalState, });
    }

    quote! {
        #[doc(hidden)]
        #vis struct #data_name #generics (
            #data_struct_fields
        );

        #vis struct #signal_state_name (
            #signal_state_fields
        );
    }
}

/// Generate the `ComponentData` implementation
pub(crate) fn generate_component_data_impl(
    generic_params: &GenericParams,
    component_params: &ComponentImplParams,
) -> TokenStream {
    let impl_generics = generic_params.impl_generics;
    let type_generics = generic_params.type_generics;
    let where_clause = generic_params.where_clause;

    let data_name = component_params.data_name;
    let signal_state_name = component_params.signal_state_name;
    let field_count = component_params.field_count;
    let fields = component_params.fields;
    let is_named = component_params.is_named;

    let signals_mut_body = {
        let mut signals_mut_body = TokenStream::new();
        for field in fields {
            let access = &field.access;
            signals_mut_body.extend(quote! { &mut self.#access, });
        }
        signals_mut_body
    };
    let pop_signals_body = generate_pop_signals_body(signal_state_name, fields, is_named);
    let set_signals_body = {
        let mut set_signals_body = TokenStream::new();
        for field in fields {
            let access = &field.access;
            set_signals_body.extend(quote! { self.#access.set_state(state.#access); });
        }
        set_signals_body
    };

    quote! {
        #[automatically_derived]
        impl #impl_generics ::natrix::macro_ref::ComponentData for #data_name #type_generics #where_clause {
            type FieldRef<'s> = [&'s mut dyn ::natrix::macro_ref::SignalMethods; #field_count];
            type SignalState = #signal_state_name;

            fn signals_mut(&mut self) -> Self::FieldRef<'_> {
                [
                    #signals_mut_body
                ]
            }

            fn pop_signals(&mut self) -> Self::SignalState {
                #pop_signals_body
            }

            fn set_signals(&mut self, state: Self::SignalState) {
                #set_signals_body
            }
        }
    }
}

/// Generate the `pop_signals` method body
pub(crate) fn generate_pop_signals_body(
    signal_state_name: &syn::Ident,
    fields: &[Field],
    is_named: bool,
) -> TokenStream {
    if is_named {
        let mut field_assignments = TokenStream::new();
        for field in fields {
            let access = &field.access;
            field_assignments.extend(quote! { #access: self.#access.pop_state(), });
        }
        quote! {
            #signal_state_name {
                #field_assignments
            }
        }
    } else {
        let mut field_pops = TokenStream::new();
        for field in fields {
            let access = &field.access;
            field_pops.extend(quote! { self.#access.pop_state(), });
        }
        quote! {
            #signal_state_name (
                #field_pops
            )
        }
    }
}

/// Generate the `ComponentBase` implementation
pub(crate) fn generate_component_base_impl(
    generic_params: &GenericParams,
    name: &syn::Ident,
    component_params: &ComponentImplParams,
) -> TokenStream {
    let impl_generics = generic_params.impl_generics;
    let type_generics = generic_params.type_generics;
    let where_clause = generic_params.where_clause;

    let data_name = component_params.data_name;
    let fields = component_params.fields;
    let is_named = component_params.is_named;

    let into_data_body = generate_into_data_body(data_name, fields, is_named);

    quote! {
        #[automatically_derived]
        impl #impl_generics ::natrix::macro_ref::ComponentBase for #name #type_generics #where_clause {
            type Data = #data_name #type_generics;
            fn into_data(self) -> Self::Data {
                #into_data_body
            }
        }
    }
}

/// Generate the `into_data` method body
pub(crate) fn generate_into_data_body(
    data_name: &syn::Ident,
    fields: &[Field],
    is_named: bool,
) -> TokenStream {
    if is_named {
        let mut field_assignments = TokenStream::new();
        for field in fields {
            let access = &field.access;
            field_assignments
                .extend(quote! { #access: ::natrix::macro_ref::Signal::new(self.#access), });
        }
        quote! {
            #data_name {
                #field_assignments
            }
        }
    } else {
        let mut field_news = TokenStream::new();
        for field in fields {
            let access = &field.access;
            field_news.extend(quote! { ::natrix::macro_ref::Signal::new(self.#access), });
        }
        quote! {
            #data_name(
                #field_news
            )
        }
    }
}

/// Retrieve abstract fields from a struct, as well as a boolean indicating whether its a named
/// struct or not (unit structs are considered named)
pub(crate) fn get_fields(fields: syn::Fields) -> (Vec<Field>, bool) {
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
