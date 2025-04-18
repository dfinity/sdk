use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::Lit;
use syn::{parse_macro_input, Attribute, Ident, ItemStruct, LitStr, Meta, MetaNameValue, Token};

struct CommandDescriptorArgs {
    path: String,
    dispatch_fn: Option<String>,
}

impl Parse for CommandDescriptorArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let metas: Punctuated<Meta, Comma> = input.parse_terminated(Meta::parse, Token![,])?;

        let mut path = None;
        let mut dispatch_fn = None;

        for meta in metas {
            match meta {
                Meta::NameValue(MetaNameValue { path: p, value, .. }) => {
                    let ident = p
                        .get_ident()
                        .ok_or_else(|| syn::Error::new(p.span(), "expected identifier"))?;

                    let lit_str = match value {
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit),
                            ..
                        }) => lit,
                        _ => return Err(input.error("expected string literal")),
                    };

                    match ident.to_string().as_str() {
                        "path" => path = Some(lit_str.value()),
                        "dispatch_fn" => dispatch_fn = Some(lit_str.value()),
                        _ => {
                            return Err(syn::Error::new(
                                ident.span(),
                                format!("unrecognized attribute `{}`", ident),
                            ))
                        }
                    }
                }
                other => {
                    return Err(input.error(format!("unsupported attribute syntax: {:?}", other)));
                }
            }
        }

        let path = path.ok_or_else(|| input.error("missing required `path` argument"))?;

        Ok(CommandDescriptorArgs {
            path: path,
            dispatch_fn,
        })
    }
}

#[proc_macro_attribute]
pub fn command_descriptor(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse attribute args with our custom struct
    let args = parse_macro_input!(attr as CommandDescriptorArgs);

    let path_value = args.path;
    let dispatch_fn_value = args.dispatch_fn.unwrap_or_else(|| "exec".to_string());

    // Parse the struct itself
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;

    // Split the path into a vector of string literals
    let path_parts: Vec<String> = path_value
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    let path_vec = path_parts.iter().map(|p| quote! { #p.to_string() });

    // Turn the dispatch_fn string into an identifier
    let dispatch_fn_ident = syn::Ident::new(&dispatch_fn_value, struct_name.span());

    // Generate the descriptor method
    let expanded = quote! {
        #input

        pub(crate) fn descriptor() -> crate::cli::descriptor::CommandDescriptor {
            use clap::{FromArgMatches, CommandFactory};
            let path = vec![#(#path_vec),*];
            let subcommand = #struct_name::command();
            let dispatch = crate::cli::descriptor::Dispatch::Function(|matches| {
                let opts = #struct_name::from_arg_matches(matches).map_err(|e| crate::cli::error::CliError(e.to_string()))?;
                #dispatch_fn_ident(&opts)
            });
            crate::cli::descriptor::CommandDescriptor {
                path,
                subcommand,
                dispatch,
            }
        }
    };

    TokenStream::from(expanded)
}
