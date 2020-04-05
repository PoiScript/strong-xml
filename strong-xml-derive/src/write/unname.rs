use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr};

pub fn write(name: &Ident, ele_name: &Ident) -> TokenStream {
    quote! {
        log::debug!(concat!("[", stringify!(#name), "] Started writing."));

        __inner.to_writer(writer)?;

        log::debug!(concat!("[", stringify!(#name), "] Finished writing."));
    }
}
