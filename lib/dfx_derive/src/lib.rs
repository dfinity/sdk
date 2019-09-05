extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
extern crate quote;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as Tokens;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};
use syn::punctuated::Punctuated;

#[proc_macro_derive(DfinityInfo)]
pub fn derive_dfinity_info(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let body = match input.data {
        Data::Enum(ref data) => {
            enum_from_ast(&data.variants)
        },
        Data::Struct(ref data) => {
            struct_from_ast(&data.fields)
        },
        Data::Union(_) => unimplemented!("doesn't derive non-enum type")            
    };
    let gen = quote! {
        impl dfx_info::DfinityInfo for #name {
            fn get_type(&self) -> dfx_info::Type {
                #body
            }
        }
    };
    //panic!(gen.to_string());
    TokenStream::from(gen)
}

fn enum_from_ast(variants: &Punctuated<syn::Variant, Token![,]>) -> Tokens {
    let variants: Vec<(String, Tokens)> = variants.iter().map(|variant| {
        let id = variant.ident.to_string();
        let ty = struct_from_ast(&variant.fields);
        (id, ty)
    }).collect();
    let tokens = variants.iter().fold(quote! {}, |tokens, (id, ty)| {
        quote! {
            #tokens
            dfx_info::Field { id: #id.to_owned(), ty: #ty },
        }
    });
    quote! { dfx_info::Type::Variant(vec![#tokens]) }
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
    let fields: Vec<(String, Tokens)> = fields.iter().enumerate().map(|(i, field)| {
        let id = match field.ident {
            Some(ref ident) => ident.to_string(),
            None => i.to_string(),
        };
        let ty = type_from_ast(&field.ty);
        (id, ty)
    }).collect();
    let tokens = fields.iter().fold(quote! { }, |tokens, (id, ty)| {
        quote! {
            #tokens
            dfx_info::Field { id: #id.to_owned(), ty: #ty },
        }
    });
    quote! { vec![#tokens] }
}

fn type_from_ast(t: &syn::Type) -> Tokens {
    quote! { dfx_info::Type::Bool }
}
