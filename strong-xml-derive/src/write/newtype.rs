use proc_macro2::TokenStream;
use quote::quote;

pub fn write(name: TokenStream) -> TokenStream {
    quote! {
        strong_xml::log_start_writing!(#name);

        __inner.to_writer(writer)?;

        strong_xml::log_finish_writing!(#name);
    }
}
