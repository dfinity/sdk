extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
extern crate quote;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as Tokens;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Generics, GenericParam};
use syn::punctuated::Punctuated;

#[proc_macro_derive(DfinityInfo)]
pub fn derive_dfinity_info(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    // TODO respect serde attributes
    let body = match input.data {
        Data::Enum(ref data) => {
            enum_from_ast(&data.variants)
        },
        Data::Struct(ref data) => {
            struct_from_ast(&data.fields)
        },
        Data::Union(_) => unimplemented!("doesn't derive union type")            
    };
    let gen = quote! {
        impl #impl_generics dfx_info::DfinityInfo for #name #ty_generics #where_clause {
            fn _ty() -> dfx_info::Type {
                #body
            }
            fn id() -> dfx_info::TypeId { dfx_info::TypeId::of::<#name #ty_generics>() }
        }
    };
    //panic!(gen.to_string());
    TokenStream::from(gen)
}

fn enum_from_ast(variants: &Punctuated<syn::Variant, Token![,]>) -> Tokens {
    let id = variants.iter().map(|variant| variant.ident.to_string());
    let ty = variants.iter().map(|variant| struct_from_ast(&variant.fields));
    quote! {
        dfx_info::Type::Variant(
            vec![
                #(dfx_info::Field {
                    id: #id.to_owned(),
                    ty: #ty }
                ),*
            ]
        )
    }
}

fn struct_from_ast(fields: &syn::Fields) -> Tokens {
    match *fields {
        syn::Fields::Named(ref fields) => {
            let fs = fields_from_ast(&fields.named);
            quote! { dfx_info::Type::Record(#fs) }
        },
        syn::Fields::Unnamed(ref fields) => {
            let fs = fields_from_ast(&fields.unnamed);
            quote! { dfx_info::Type::Record(#fs) }
        },
        syn::Fields::Unit => quote! { dfx_info::Type::Null },
    }
}

fn fields_from_ast(fields: &Punctuated<syn::Field, syn::Token![,]>) -> Tokens {
    let id = fields.iter().enumerate().map(|(i, field)| {
        match field.ident {
            Some(ref ident) => ident.to_string(),
            None => i.to_string()
        }
    });
    let ty = fields.iter().map(|field| { derive_type(&field.ty) });
    quote! {
        vec![
            #(dfx_info::Field {
                id: #id.to_owned(),
                ty: #ty }
            ),*
        ]
    }
}

fn derive_type(t: &syn::Type) -> Tokens {
    quote! {
        <#t as dfx_info::DfinityInfo>::ty()
    }
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            let bound = syn::parse_str("::dfx_info::DfinityInfo").unwrap();
            let lifetime = syn::parse_str("'static").unwrap();
            type_param.bounds.push(bound);
            type_param.bounds.push(lifetime);
        }
    }
    generics
}
