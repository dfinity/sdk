use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct, LitStr, Ident, Meta, MetaNameValue, Attribute, Token};
use syn::parse::{Parse, ParseStream, Result};

// CommandDescriptorArgs: parses #[command_descriptor(path = "foo", dispatch_fn = "bar")]
struct CommandDescriptorArgs {
    path: LitStr,
    dispatch_fn: LitStr,
}

impl Parse for CommandDescriptorArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let path_ident: Ident = input.parse()?;
        if path_ident != "path" {
            return Err(input.error("expected `path`"));
        }
        input.parse::<Token![=]>()?;
        let path: LitStr = input.parse()?;

        input.parse::<Token![,]>()?;

        let dispatch_fn_ident: Ident = input.parse()?;
        if dispatch_fn_ident != "dispatch_fn" {
            return Err(input.error("expected `dispatch_fn`"));
        }
        input.parse::<Token![=]>()?;
        let dispatch_fn: LitStr = input.parse()?;

        Ok(CommandDescriptorArgs { path, dispatch_fn })
    }
}

#[proc_macro_attribute]
pub fn command_descriptor(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse attribute args with our custom struct
    let args = parse_macro_input!(attr as CommandDescriptorArgs);

    let path_value = args.path.value();
    let dispatch_fn_value = args.dispatch_fn.value();

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

    // Generate the impl block
    let expanded = quote! {
        #input

        impl #struct_name {
            pub(crate) fn descriptor() -> CommandDescriptor {
                let path = vec![#(#path_vec),*];
                let subcommand = Self::command();
                let dispatch = Dispatch::Function(|matches| {
                    let opts = Self::from_arg_matches(matches).map_err(|e| CliError(e.to_string()))?;
                    #dispatch_fn_ident(&opts)
                });
                CommandDescriptor {
                    path,
                    subcommand,
                    dispatch,
                }
            }
        }
    };

    TokenStream::from(expanded)
}
