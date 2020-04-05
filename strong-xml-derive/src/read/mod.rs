mod named;
mod unit;
mod unname;

use crate::types::{Element, Fields};

use proc_macro2::TokenStream;
use quote::quote;

pub fn impl_read(element: Element) -> TokenStream {
    match element {
        Element::Enum {
            name: ele_name,
            variants,
        } => {
            let tags = variants.iter().map(|variant| match variant {
                Fields::Unname { tags, .. } => tags.clone(),
                Fields::Named { tag, .. } | Fields::Unit { tag, .. } => vec![tag.clone()],
            });

            let read = variants.iter().map(|variant| match variant {
                Fields::Named { tag, name, fields } => {
                    named::read(&tag, quote!(#ele_name::#name), &fields)
                }
                Fields::Unname { name, ty, .. } => unname::read(&ty, quote!(#ele_name::#name)),
                Fields::Unit { tag, name } => unit::read(&tag, quote!(#ele_name::#name)),
            });

            quote! {
                while let Some(tag) = reader.find_element_start(None)? {
                    match tag {
                        #( #( #tags )|* => { #read } )*
                        tag => {
                            log::info!(
                                concat!("[", stringify!(#ele_name), "] Skip element `{}`"),
                                tag
                            );
                            // skip the start tag
                            reader.next();
                            reader.read_to_end(tag)?;
                        },
                    }
                }

                Err(XmlError::UnexpectedEof)
            }
        }

        Element::Struct { fields, .. } => match fields {
            Fields::Named { tag, name, fields } => named::read(&tag, quote!(#name).into(), &fields),
            Fields::Unname { name, ty, .. } => unname::read(&ty, quote!(#name).into()),
            Fields::Unit { tag, name } => unit::read(&tag, quote!(#name).into()),
        },
    }
}
