use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr};

pub fn read(name: &Ident, ele_name: &Ident, path: Option<TokenStream>) -> TokenStream {
    quote! {
        log::debug!(concat!("[", stringify!(#ele_name), "] Started reading"));

        let res = #name::from_reader(reader)?;

        log::debug!(concat!("[", stringify!(#ele_name), "] Finished reading"));

        return Ok(#path(res));
    }
}
