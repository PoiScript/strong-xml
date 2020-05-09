#![recursion_limit = "256"]

extern crate proc_macro;

mod read;
mod types;
mod utils;
mod write;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use types::Element;

#[proc_macro_derive(XmlRead, attributes(xml))]
pub fn derive_xml_read(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;

    let input_lifetime = utils::gen_input_lifetime(&generics);

    let mut generics_with_lifetime_and_where = generics.clone();
    generics_with_lifetime_and_where
        .params
        .push(input_lifetime.into());
    generics_with_lifetime_and_where
        .make_where_clause()
        .predicates
        .extend(generics.type_params().map(|t| {
            let x: syn::WherePredicate = syn::parse_quote! {#t: strong_xml::XmlRead<'__input>};
            x
        }));

    let (_, ty_generics, _) = generics.split_for_impl();
    let (impl_generics, _, where_clause) = generics_with_lifetime_and_where.split_for_impl();

    let impl_read = read::impl_read(Element::parse(input.clone()));

    let gen = quote! {
        impl #impl_generics strong_xml::XmlRead<'__input> for #name #ty_generics #where_clause {
            fn from_reader(
                mut reader: &mut strong_xml::XmlReader<'__input>
            ) -> strong_xml::XmlResult<Self> {
                use strong_xml::xmlparser::{ElementEnd, Token, Tokenizer};
                use strong_xml::XmlError;
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
    let mut generics_with_where = generics.clone();

    generics_with_where
        .make_where_clause()
        .predicates
        .extend(generics.type_params().map(|t| {
            let x: syn::WherePredicate = syn::parse_quote! {#t: strong_xml::XmlWrite};
            x
        }));

    let (impl_generics, ty_generics, where_clause) = generics_with_where.split_for_impl();

    let impl_write = write::impl_write(Element::parse(input.clone()));

    let gen = quote! {
        impl #impl_generics strong_xml::XmlWrite for #name #ty_generics #where_clause {
            fn to_writer<W: std::io::Write>(
                &self,
                mut writer: &mut strong_xml::XmlWriter<W>
            ) -> strong_xml::XmlResult<()> {
                #impl_write

                Ok(())
            }
        }
    };

    gen.into()
}
