use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr};

pub fn read(
    tag: &LitStr,
    name: TokenStream,
    ele_name: &Ident,
    path: Option<TokenStream>,
) -> TokenStream {
    quote! {
        log::debug!(concat!("[", stringify!(#ele_name), "] Started reading"));

        reader.read_till_element_start(#tag)?;

        reader.read_to_end(#tag)?;

        log::debug!(concat!("[", stringify!(#ele_name), "] Finished reading"));

        return Ok(#path(#name));
    }
}
