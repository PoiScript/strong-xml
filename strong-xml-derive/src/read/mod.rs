mod named;
mod newtype;

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
                Fields::Newtype { tags, .. } => tags.clone(),
                Fields::Named { tag, .. } => vec![tag.clone()],
            });

            let read = variants.iter().map(|variant| match variant {
                Fields::Named {
                    tag,
                    name,
                    fields,
                    prefix,
                    namespaces,
                } => named::read(&tag, quote!(#ele_name::#name), &fields),
                Fields::Newtype { name, ty, .. } => newtype::read(&ty, quote!(#ele_name::#name)),
            });

            quote! {
                while let Some(tag) = reader.find_element_start(None)? {
                    match tag {
                        #( #( #tags )|* => { #read } )*
                        tag => {
                            strong_xml::log_skip_element!(#ele_name, tag);
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
            Fields::Named {
                tag,
                name,
                fields,
                prefix,
                namespaces,
            } => named::read(&tag, quote!(#name), &fields),
            Fields::Newtype { name, ty, .. } => newtype::read(&ty, quote!(#name)),
        },
    }
}
