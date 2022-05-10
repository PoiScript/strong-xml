mod named;
mod newtype;

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
                    let bindings = fields.iter().map(|field| match field {
                        Field::Attribute { bind, name, .. }
                        | Field::Child { bind, name, .. }
                        | Field::Text { bind, name, .. }
                        | Field::FlattenText { bind, name, .. } => quote!( #name: #bind ),
                    });
                    quote!( #ele_name::#name { #( #bindings ),* } )
                }
                Fields::Newtype { name, .. } => quote!( #ele_name::#name(__inner) ),
            });

            let read = variants.iter().map(|variant| match variant {
                Fields::Named {
                    tag,
                    name,
                    fields,
                    prefix,
                    namespaces,
                } => named::write(tag, prefix, quote!( #ele_name::#name ), &fields, &namespaces),
                Fields::Newtype { name, .. } => newtype::write(quote!( #ele_name::#name )),
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
            Fields::Named {
                tag,
                name,
                fields,
                prefix,
                namespaces,
            } => {
                let bindings = fields.iter().map(|field| match field {
                    Field::Attribute { bind, name, .. }
                    | Field::Child { bind, name, .. }
                    | Field::Text { bind, name, .. }
                    | Field::FlattenText { bind, name, .. } => quote!( #name: #bind ),
                });

                let read = named::write(&tag, &prefix, quote!(#name), &fields, &namespaces);

                quote! {
                    let #ele_name { #( #bindings ),* } = self;

                    #read
                }
            }
            Fields::Newtype { name, .. } => {
                let read = newtype::write(quote!(#name));

                quote! {
                    let __inner = &self.0;

                    #read
                }
            }
        },
    }
}
