//! Derive macros for [Natrix](https://github.com/vivax3794/natrix)

extern crate proc_macro;

mod formatting;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::{fs, io};

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::parse_quote;

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

/// Derive the `Project` trait for an enum
#[proc_macro_derive(Project, attributes(project))]
pub fn project_derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(item as syn::ItemEnum);
    let name = &item.ident;
    let generics = &item.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    // Parse additional attributes from #[project(...)] 
    let mut additional_attrs = Vec::new();
    for attr in &item.attrs {
        if attr.path().is_ident("project") {
            if let syn::Meta::List(meta_list) = &attr.meta {
                // Parse the meta list for derive(...) and other attributes
                let nested = meta_list.parse_args_with(syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated);
                if let Ok(nested_metas) = nested {
                    for meta in nested_metas {
                        match meta {
                            syn::Meta::List(list) if list.path.is_ident("derive") => {
                                // Handle derive(...) specially
                                let derive_tokens = &list.tokens;
                                additional_attrs.push(quote!(#[derive(#derive_tokens)]));
                            }
                            _ => {
                                // Handle other attributes like serde(...)
                                additional_attrs.push(quote!(#[#meta]));
                            }
                        }
                    }
                }
            }
        }
    }
    let additional_derives = quote!(#(#additional_attrs)*);

    // Check if this is a pure-unit enum (all variants are unit variants)
    let has_fields = item.variants.iter().any(|variant| {
        !matches!(variant.fields, syn::Fields::Unit)
    });

    if !has_fields {
        // Pure-unit enums break invariants with FaillableMut(None)
        return quote! {
            compile_error!("Cannot derive Project for pure-unit enums (enums with only unit variants). Project derive requires at least one variant with fields to maintain FaillableMut(None) invariants.")
        }.into();
    }

    // Generate projected enum generics with lifetime only if needed
    let needs_lifetime = has_fields;

    // Generate the projected enum
    let projected_name = format_ident!("{}Projected", name);
    let mut projected_variants = Vec::new();
    let mut read_arms = Vec::new();
    let mut mut_arms = Vec::new();
    let mut faillable_none_arms = Vec::new();
    let mut faillable_some_arms = Vec::new();

    for variant in &item.variants {
        let variant_name = &variant.ident;
        match &variant.fields {
            syn::Fields::Named(fields) => {
                let field_names: Vec<_> = fields.named.iter().map(|f| f.ident.as_ref().unwrap()).collect();
                let field_types: Vec<_> = fields.named.iter().map(|f| &f.ty).collect();
                
                projected_variants.push(quote! {
                    #variant_name { #(#field_names: ::natrix::access::Ref<'a, #field_types>),* }
                });

                read_arms.push(quote! {
                    #name::#variant_name { #(#field_names),* } => #projected_name::#variant_name { #(#field_names: ::natrix::access::Ref::Read(#field_names)),* }
                });

                mut_arms.push(quote! {
                    #name::#variant_name { #(#field_names),* } => #projected_name::#variant_name { #(#field_names: ::natrix::access::Ref::Mut(#field_names)),* }
                });

                if fields.named.is_empty() {
                    faillable_none_arms.push(quote! {
                        #projected_name::#variant_name
                    });
                } else {
                    let none_field_inits: Vec<_> = field_names.iter().map(|name| quote!(#name: ::natrix::access::Ref::FaillableMut(None))).collect();
                    faillable_none_arms.push(quote! {
                        #projected_name::#variant_name { #(#none_field_inits),* }
                    });
                }

                faillable_some_arms.push(quote! {
                    #name::#variant_name { #(#field_names),* } => #projected_name::#variant_name { #(#field_names: ::natrix::access::Ref::FaillableMut(Some(#field_names))),* }
                });
            },
            syn::Fields::Unnamed(fields) => {
                let field_types: Vec<_> = fields.unnamed.iter().map(|f| &f.ty).collect();
                let field_names: Vec<_> = (0..fields.unnamed.len()).map(|i| format_ident!("field_{}", i)).collect();
                
                projected_variants.push(quote! {
                    #variant_name(#(::natrix::access::Ref<'a, #field_types>),*)
                });

                read_arms.push(quote! {
                    #name::#variant_name(#(#field_names),*) => #projected_name::#variant_name(#(::natrix::access::Ref::Read(#field_names)),*)
                });

                mut_arms.push(quote! {
                    #name::#variant_name(#(#field_names),*) => #projected_name::#variant_name(#(::natrix::access::Ref::Mut(#field_names)),*)
                });

                if fields.unnamed.is_empty() {
                    faillable_none_arms.push(quote! {
                        #projected_name::#variant_name
                    });
                } else {
                    let none_refs: Vec<_> = (0..fields.unnamed.len()).map(|_| quote!(::natrix::access::Ref::FaillableMut(None))).collect();
                    faillable_none_arms.push(quote! {
                        #projected_name::#variant_name(#(#none_refs),*)
                    });
                }

                faillable_some_arms.push(quote! {
                    #name::#variant_name(#(#field_names),*) => #projected_name::#variant_name(#(::natrix::access::Ref::FaillableMut(Some(#field_names))),*)
                });
            },
            syn::Fields::Unit => {
                projected_variants.push(quote! {
                    #variant_name
                });

                read_arms.push(quote! {
                    #name::#variant_name => #projected_name::#variant_name
                });

                mut_arms.push(quote! {
                    #name::#variant_name => #projected_name::#variant_name
                });

                faillable_none_arms.push(quote! {
                    #projected_name::#variant_name
                });

                faillable_some_arms.push(quote! {
                    #name::#variant_name => #projected_name::#variant_name
                });
            }
        }
    }

    // For FaillableMut(None), we need to return a valid variant that contains refs
    // Find the first variant that contains fields (not a unit variant)
    let first_variant_with_fields = item.variants.iter().enumerate().find_map(|(idx, variant)| {
        match &variant.fields {
            syn::Fields::Unit => None,
            _ => Some(idx),
        }
    });
    
    let default_faillable_none = quote!(unreachable!());
    let first_faillable_none = if let Some(field_variant_idx) = first_variant_with_fields {
        faillable_none_arms.get(field_variant_idx).unwrap_or(&default_faillable_none)
    } else {
        // If no variants have fields, just use the first variant (unit variants are fine)
        faillable_none_arms.first().unwrap_or(&default_faillable_none)
    };

    // Generate the final TokenStream
    let (projected_enum_def, projected_type_for_trait) = if needs_lifetime {
        let mut projected_generics = item.generics.clone();
        let lifetime_param = syn::GenericParam::Lifetime(syn::LifetimeParam {
            attrs: vec![],
            lifetime: syn::Lifetime::new("'a", proc_macro2::Span::call_site()),
            colon_token: None,
            bounds: syn::punctuated::Punctuated::new(),
        });
        projected_generics.params.insert(0, lifetime_param);
        let (proj_impl_generics, proj_type_generics, proj_where_clause) = projected_generics.split_for_impl();
        
        (quote! {
            #additional_derives
            #[automatically_derived]
            pub enum #projected_name #proj_impl_generics #proj_where_clause {
                #(#projected_variants),*
            }
        }, proj_type_generics.to_token_stream())
    } else {
        let (proj_impl_generics, proj_type_generics, proj_where_clause) = generics.split_for_impl();
        (quote! {
            #additional_derives
            #[automatically_derived]
            pub enum #projected_name #proj_impl_generics #proj_where_clause {
                #(#projected_variants),*
            }
        }, proj_type_generics.to_token_stream())
    };

    quote! {
        #projected_enum_def

        #[automatically_derived]
        impl #impl_generics ::natrix::access::Project for #name #type_generics #where_clause {
            type Projected<'a> = #projected_name #projected_type_for_trait where Self: 'a;

            fn project(value: ::natrix::access::Ref<'_, Self>) -> Self::Projected<'_> {
                match value {
                    ::natrix::access::Ref::Read(v) => match v {
                        #(#read_arms),*
                    },
                    ::natrix::access::Ref::Mut(v) => match v {
                        #(#mut_arms),*
                    },
                    ::natrix::access::Ref::FaillableMut(None) => {
                        #first_faillable_none
                    },
                    ::natrix::access::Ref::FaillableMut(Some(v)) => match v {
                        #(#faillable_some_arms),*
                    },
                }
            }
        }
    }
    .into()
}

/// Derive the `ProjectIntoState` trait for an enum to enable ProjectableSignal usage
#[proc_macro_derive(ProjectIntoState)]
pub fn project_into_state_derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(item as syn::ItemEnum);
    let name = &item.ident;
    let generics = &item.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_generics ::natrix::reactivity::signal::ProjectIntoState for #name #type_generics #where_clause {}
    }
    .into()
}

/// Derive the `Downgrade` trait for an enum
#[proc_macro_derive(Downgrade)]
pub fn downgrade_derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(item as syn::ItemEnum);
    let name = &item.ident;
    let generics = &item.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    // Build where clause for Downgrade bounds on all field types
    let mut downgrade_where_clause = if let Some(where_clause) = where_clause {
        quote! { #where_clause, }
    } else {
        quote! { where }
    };

    // Collect all field types that need Downgrade bounds - use strings for deduplication
    let mut field_type_strings = std::collections::HashSet::new();
    for variant in &item.variants {
        match &variant.fields {
            syn::Fields::Named(fields) => {
                for field in &fields.named {
                    field_type_strings.insert(field.ty.to_token_stream().to_string());
                }
            },
            syn::Fields::Unnamed(fields) => {
                for field in &fields.unnamed {
                    field_type_strings.insert(field.ty.to_token_stream().to_string());
                }
            },
            syn::Fields::Unit => {}
        }
    }

    for field_type_str in &field_type_strings {
        let field_type: syn::Type = syn::parse_str(field_type_str).unwrap();
        downgrade_where_clause = quote! { #downgrade_where_clause #field_type: ::natrix::access::Downgrade<'a>, };
    }

    // Generate ReadOutput and MutOutput enum variants
    let read_output_name = format_ident!("{}ReadOutput", name);
    let mut_output_name = format_ident!("{}MutOutput", name);
    let mut read_output_variants = Vec::new();
    let mut mut_output_variants = Vec::new();
    let mut into_read_arms = Vec::new();
    let mut into_mut_arms = Vec::new();

    for variant in &item.variants {
        let variant_name = &variant.ident;
        match &variant.fields {
            syn::Fields::Named(fields) => {
                let field_names: Vec<_> = fields.named.iter().map(|f| f.ident.as_ref().unwrap()).collect();
                let field_types: Vec<_> = fields.named.iter().map(|f| &f.ty).collect();
                
                read_output_variants.push(quote! {
                    #variant_name { #(#field_names: <#field_types as ::natrix::access::Downgrade<'a>>::ReadOutput),* }
                });
                
                mut_output_variants.push(quote! {
                    #variant_name { #(#field_names: <#field_types as ::natrix::access::Downgrade<'a>>::MutOutput),* }
                });
                
                into_read_arms.push(quote! {
                    Self::#variant_name { #(#field_names),* } => {
                        Some(#read_output_name::#variant_name {
                            #(#field_names: #field_names.into_read()?),*
                        })
                    }
                });

                into_mut_arms.push(quote! {
                    Self::#variant_name { #(#field_names),* } => {
                        Some(#mut_output_name::#variant_name {
                            #(#field_names: #field_names.into_mut()?),*
                        })
                    }
                });
            },
            syn::Fields::Unnamed(fields) => {
                let field_names: Vec<_> = (0..fields.unnamed.len()).map(|i| format_ident!("field_{}", i)).collect();
                let field_types: Vec<_> = fields.unnamed.iter().map(|f| &f.ty).collect();
                
                read_output_variants.push(quote! {
                    #variant_name(#(<#field_types as ::natrix::access::Downgrade<'a>>::ReadOutput),*)
                });
                
                mut_output_variants.push(quote! {
                    #variant_name(#(<#field_types as ::natrix::access::Downgrade<'a>>::MutOutput),*)
                });
                
                into_read_arms.push(quote! {
                    Self::#variant_name(#(#field_names),*) => {
                        Some(#read_output_name::#variant_name(
                            #(#field_names.into_read()?),*
                        ))
                    }
                });

                into_mut_arms.push(quote! {
                    Self::#variant_name(#(#field_names),*) => {
                        Some(#mut_output_name::#variant_name(
                            #(#field_names.into_mut()?),*
                        ))
                    }
                });
            },
            syn::Fields::Unit => {
                read_output_variants.push(quote! {
                    #variant_name
                });
                
                mut_output_variants.push(quote! {
                    #variant_name
                });
                
                into_read_arms.push(quote! {
                    Self::#variant_name => Some(#read_output_name::#variant_name)
                });

                into_mut_arms.push(quote! {
                    Self::#variant_name => Some(#mut_output_name::#variant_name)
                });
            }
        }
    }

    quote! {
        // Generate the ReadOutput enum
        #[automatically_derived]
        pub enum #read_output_name #impl_generics #where_clause {
            #(#read_output_variants),*
        }
        
        // Generate the MutOutput enum
        #[automatically_derived]
        pub enum #mut_output_name #impl_generics #where_clause {
            #(#mut_output_variants),*
        }

        #[automatically_derived]
        impl<'a> ::natrix::access::Downgrade<'a> for #name #type_generics #downgrade_where_clause {
            type ReadOutput = #read_output_name #type_generics;
            type MutOutput = #mut_output_name #type_generics;

            fn into_read(self) -> Option<Self::ReadOutput> {
                match self {
                    #(#into_read_arms),*
                }
            }

            fn into_mut(self) -> Option<Self::MutOutput> {
                match self {
                    #(#into_mut_arms),*
                }
            }
        }
    }
    .into()
}

/// Derive the `State` trait for a struct
#[proc_macro_derive(State)]
pub fn state_derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(item as syn::ItemStruct);
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
    let mut set_statements = quote!();

    for field in &fields {
        let type_ = &field.type_;
        let access = &field.access;

        where_clause = quote!(#where_clause #type_: ::natrix::macro_ref::State ,);
        set_statements = quote!(#set_statements self.#access.set(new.#access););
    }

    quote! {
        #[automatically_derived]
        impl #impl_generics ::natrix::macro_ref::State for #name #type_generics #where_clause {
            fn set(&mut self, new: Self) {
                #set_statements
            }
        }
    }
    .into()
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

/// A abstract representation of a struct field
struct Field {
    /// The type of the field
    type_: TokenStream,
    /// How to access the field
    access: TokenStream,
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
                access: field.ident.to_token_stream(),
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
