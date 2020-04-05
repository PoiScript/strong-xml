mod named;
mod unit;
mod unname;

use crate::types::{Element, Fields};

use proc_macro2::TokenStream;
use quote::quote;

pub fn impl_read(element: Element) -> TokenStream {
    match element {
        Element::Enum { name, variants } => {
            let tags = variants.iter().map(|variant| match variant {
                Fields::Unname { tags, .. } => tags.clone(),
                Fields::Named { tag, .. } | Fields::Unit { tag, .. } => vec![tag.clone()],
            });

            let read = variants.iter().map(|variant| {
                let var_name = match variant {
                    Fields::Named { name, .. }
                    | Fields::Unname { name, .. }
                    | Fields::Unit { name, .. } => name,
                };

                let path = quote!(#name::#var_name);

                match variant {
                    Fields::Named { tag, name, fields } => {
                        named::read(&tag, &name, &fields, Some(path))
                    }
                    Fields::Unname { name, ty, .. } => unname::read(&name, &name, Some(path)),
                    Fields::Unit { tag, name } => unit::read(&tag, path.into(), &name, None),
                }
            });

            quote! {
                while let Some(tag) = reader.find_element_start(None)? {
                    match tag {
                        #( #( #tags )|* => { #read } )*
                        tag => {
                            log::info!(
                                concat!("[", stringify!(#name), "] Skip element `{}`"),
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

        Element::Struct { name, fields } => match fields {
            Fields::Named { tag, name, fields } => named::read(&tag, &name, &fields, None),
            Fields::Unname { name, ty, .. } => unname::read(&name, &name, None),
            Fields::Unit { tag, name } => unit::read(&tag, quote!(#name).into(), &name, None),
        },
    }
}
