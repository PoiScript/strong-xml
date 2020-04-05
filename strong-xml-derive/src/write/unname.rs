use proc_macro2::TokenStream;
use quote::quote;

pub fn write(name: TokenStream) -> TokenStream {
    quote! {
        log::debug!(concat!("[", stringify!(#name), "] Started writing."));

        __inner.to_writer(writer)?;

        log::debug!(concat!("[", stringify!(#name), "] Finished writing."));
    }
}
