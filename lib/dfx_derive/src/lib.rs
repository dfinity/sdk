extern crate proc_macro;
extern crate syn;
extern crate quote;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics, Index};

#[proc_macro_derive(DfinityInfo)]
pub fn derive_dfinity_info(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let gen = quote! {
        impl dfx_info::DfinityInfo for #name {
            fn get_type(&self) -> dfx_info::Type {
                dfx_info::Type::Nat
            }
        }
    };
    TokenStream::from(gen)
}
