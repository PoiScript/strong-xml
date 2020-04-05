mod named;
mod unit;
mod unname;

use crate::types::{Element, Field, Fields};

use proc_macro2::TokenStream;
use quote::quote;

pub fn impl_write(element: Element) -> TokenStream {
    match element {
        Element::Enum {
            name: ele_name,
            variants,
        } => {
            let branches = variants.iter().map(|variant| match variant {
                Fields::Named { name, fields, .. } => {
                    let names = fields.iter().map(|field| match field {
                        Field::Attribute { name, .. }
                        | Field::Child { name, .. }
                        | Field::Text { name, .. }
                        | Field::FlattenText { name, .. } => name,
                    });
                    quote!( #ele_name::#name { #( #names ),* } )
                }
                Fields::Unname { name, .. } => quote!( #ele_name::#name(__inner) ),
                Fields::Unit { name, .. } => quote!( #ele_name::#name ),
            });

            let read = variants.iter().map(|variant| match variant {
                Fields::Named { tag, name, fields } => {
                    named::write(&tag, quote!( #ele_name::#name ), &fields)
                }
                Fields::Unname { name, .. } => unname::write(quote!( #ele_name::#name )),
                Fields::Unit { tag, name } => unit::write(&tag, quote!( #ele_name::#name )),
            });

            quote! {
                match self {
                    #( #branches => { #read }, )*
                }
            }
        }

        Element::Struct {
            name: ele_name,
            fields,
        } => match fields {
            Fields::Named { tag, name, fields } => {
                let names = fields.iter().map(|field| match field {
                    Field::Attribute { name, .. }
                    | Field::Child { name, .. }
                    | Field::Text { name, .. }
                    | Field::FlattenText { name, .. } => name,
                });

                let read = named::write(&tag, quote!(#name), &fields);

                quote! {
                    let #ele_name { #( #names ),* } = self;

                    #read
                }
            }
            Fields::Unname { name, .. } => {
                let read = unname::write(quote!(#name));

                quote! {
                    let __inner = &self.0;

                    #read
                }
            }
            Fields::Unit { tag, name } => unit::write(&tag, quote!(#name)),
        },
    }
}
