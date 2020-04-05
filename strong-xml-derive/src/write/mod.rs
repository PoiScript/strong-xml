mod named;
mod unit;
mod unname;

use crate::types::{Element, Field, Fields};

use proc_macro2::TokenStream;
use quote::quote;

pub fn impl_write(element: Element) -> TokenStream {
    match element {
        Element::Enum { name, variants } => {
            let names = variants.iter().map(|variant| match variant {
                Fields::Named { name, fields, .. } => {
                    let names = fields.iter().map(|field| match field {
                        Field::Attribute { name, .. }
                        | Field::Child { name, .. }
                        | Field::Text { name, .. }
                        | Field::FlattenText { name, .. } => name,
                    });
                    quote!( #name { #( #names ),* } )
                }
                Fields::Unname { name, .. } => quote!( #name(__inner) ),
                Fields::Unit { name, .. } => quote!( #name ),
            });

            let a = variants.iter().map(|variant| match variant {
                Fields::Named { tag, name, fields } => named::write(&tag, &name, &fields),
                Fields::Unname { name, .. } => unname::write(&name, &name),
                Fields::Unit { tag, name } => unit::write(&name, &tag, &name),
            });

            quote! {
                match self {
                    #( #name::#names => { #a }, )*
                }
            }
        }

        Element::Struct { name: struct_name, fields } => match fields {
            Fields::Named { tag, name, fields } => {
                let names = fields.iter().map(|field| match field {
                    Field::Attribute { name, .. }
                    | Field::Child { name, .. }
                    | Field::Text { name, .. }
                    | Field::FlattenText { name, .. } => name,
                });

                let a = named::write(&tag, &name, &fields);

                quote! {
                    let #struct_name { #( #names ),* } = self;

                    #a
                }
            }
            Fields::Unname { name, ty, .. } => unname::write(&name, &name),
            Fields::Unit { tag, name } => unit::write(&name, &tag, &name),
        },
    }
}
