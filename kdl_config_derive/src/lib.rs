use proc_macro::{self, TokenStream};
use syn::parse_macro_input;

mod kdl_config;
mod kdl_config_finalize;

#[proc_macro_derive(KdlConfigFinalize, attributes(kdl_config_finalize_into))]
pub fn kdl_config_finalize(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input);
    kdl_config_finalize::generate(derive_input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(KdlConfig)]
pub fn kdl_config(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input);
    kdl_config::generate(derive_input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
