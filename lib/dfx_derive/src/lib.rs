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
use std::collections::BTreeSet;

#[proc_macro_derive(IDLType)]
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
        impl #impl_generics dfx_info::IDLType for #name #ty_generics #where_clause {
            fn _ty() -> dfx_info::types::Type {
                #body
            }
            fn id() -> dfx_info::types::TypeId { dfx_info::types::TypeId::of::<#name #ty_generics>() }
        }
    };
    //panic!(gen.to_string());
    TokenStream::from(gen)
}

#[inline]
fn idl_hash(id: &str) -> u32 {
    let mut s: u32 = 0;
    for c in id.chars() {
        s = s.wrapping_mul(223).wrapping_add(c as u32);
    }
    s
}

fn enum_from_ast(variants: &Punctuated<syn::Variant, Token![,]>) -> Tokens {
    let mut fs: Vec<_> = variants.iter().map(|variant| {
        let id = variant.ident.to_string();
        let hash = idl_hash(&id);
        let ty = struct_from_ast(&variant.fields);
        (id, hash, ty)
    }).collect();
    let unique: BTreeSet<_> = fs.iter().map(|(_,hash,_)| hash).collect();
    assert_eq!(unique.len(), fs.len());
    fs.sort_unstable_by_key(|(_,hash,_)| hash.clone());
    let id = fs.iter().map(|(id,_,_)| id);
    let hash = fs.iter().map(|(_,hash,_)| hash);
    let ty = fs.iter().map(|(_,_,ty)| ty);
    quote! {
        dfx_info::types::Type::Variant(
            vec![
                #(dfx_info::types::Field {
                    id: #id.to_owned(),
                    hash: #hash,
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
            quote! { dfx_info::types::Type::Record(#fs) }
        },
        syn::Fields::Unnamed(ref fields) => {
            let fs = fields_from_ast(&fields.unnamed);
            quote! { dfx_info::types::Type::Record(#fs) }
        },
        syn::Fields::Unit => quote! { dfx_info::types::Type::Null },
    }
}

fn fields_from_ast(fields: &Punctuated<syn::Field, syn::Token![,]>) -> Tokens {
    let mut fs: Vec<_> = fields.iter().enumerate().map(|(i, field)| {
        let (id, hash) = match field.ident {
            Some(ref ident) => (ident.to_string(), idl_hash(&ident.to_string())),
            None => (i.to_string(), i as u32),
        };
        let ty = derive_type(&field.ty);
        (id, hash, ty)
    }).collect();
    let unique: BTreeSet<_> = fs.iter().map(|(_,hash,_)| hash).collect();
    assert_eq!(unique.len(), fs.len());
    fs.sort_unstable_by_key(|(_,hash,_)| hash.clone());
    let id = fs.iter().map(|(id,_,_)| id);
    let hash = fs.iter().map(|(_,hash,_)| hash);
    let ty = fs.iter().map(|(_,_,ty)| ty);
    quote! {
        vec![
            #(dfx_info::types::Field {
                id: #id.to_owned(),
                hash: #hash,
                ty: #ty }
            ),*
        ]
    }
}

fn derive_type(t: &syn::Type) -> Tokens {
    quote! {
        <#t as dfx_info::IDLType>::ty()
    }
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            let bound = syn::parse_str("::dfx_info::IDLType").unwrap();
            type_param.bounds.push(bound);
        }
    }
    generics
}
