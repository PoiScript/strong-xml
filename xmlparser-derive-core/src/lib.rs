#![recursion_limit = "256"]

extern crate proc_macro;

mod types;
mod xml_write;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use crate::{types::Element, xml_write::write};

#[proc_macro_derive(XmlWrite, attributes(xml))]
pub fn derive_xml_write(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;

    let element = Element::parse(&input);

    let impl_write = write(&element);

    let gen = quote! {
        impl #generics #name #generics {
            fn to_string(&self) -> xmlparser_derive_utils::XmlResult<String> {
                let mut writer = vec![];

                self.to_writer(&mut writer)?;

                Ok(String::from_utf8(writer)?)
            }

            fn to_writer<W: std::io::Write>(
                &self,
                mut writer: W
            ) -> xmlparser_derive_utils::XmlResult<()> {
                #impl_write

                Ok(())
            }
        }
    };

    gen.into()
}
