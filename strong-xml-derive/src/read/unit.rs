use proc_macro2::TokenStream;
use quote::quote;
use syn::LitStr;

pub fn read(tag: &LitStr, ele_name: TokenStream) -> TokenStream {
    quote! {
        log::debug!(concat!("[", stringify!(#ele_name), "] Started reading"));

        reader.read_till_element_start(#tag)?;

        reader.read_to_end(#tag)?;

        log::debug!(concat!("[", stringify!(#ele_name), "] Finished reading"));

        return Ok(#ele_name);
    }
}
