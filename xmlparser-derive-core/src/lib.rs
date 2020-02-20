#![recursion_limit = "256"]

extern crate proc_macro;

mod types;
mod xml_read;
mod xml_write;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, GenericParam};

use crate::{types::Element, xml_read::read, xml_write::write};

#[proc_macro_derive(XmlRead, attributes(xml))]
pub fn derive_xml_read(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;

    let lifetime = match &generics.params.last() {
        Some(GenericParam::Lifetime(lt)) => Some(lt),
        _ => None,
    };

    let element = Element::parse(&input);

    let impl_read = read(&element);

    let gen = quote! {
        impl #generics #name #generics {
            fn from_str(
                string: & #lifetime str
            ) -> xmlparser_derive::XmlResult<#name #generics> {
                let mut reader = xmlparser::Tokenizer::from(string).peekable();
                Self::from_reader(&mut reader)
            }

            fn from_reader(
                mut reader: &mut xmlparser_derive::XmlReader #generics
            ) -> xmlparser_derive::XmlResult<#name #generics> {
                use xmlparser::{ElementEnd, Token, Tokenizer};
                use xmlparser_derive::XmlError;

                #impl_read
            }
        }
    };

    gen.into()
}

#[proc_macro_derive(XmlWrite, attributes(xml))]
pub fn derive_xml_write(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;

    let element = Element::parse(&input);

    let impl_write = write(&element);

    let gen = quote! {
        impl #generics #name #generics {
            fn to_string(&self) -> xmlparser_derive::XmlResult<String> {
                let mut writer = vec![];

                self.to_writer(&mut writer)?;

                Ok(String::from_utf8(writer)?)
            }

            fn to_writer<W: std::io::Write>(
                &self,
                mut writer: W
            ) -> xmlparser_derive::XmlResult<()> {
                #impl_write

                Ok(())
            }
        }
    };

    gen.into()
}
