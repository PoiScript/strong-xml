mod named;
mod newtype;

use crate::types::{Element, Fields};

use proc_macro2::TokenStream;
use quote::quote;
use syn::LitStr;

pub fn impl_read(element: Element) -> TokenStream {
    match element {
        Element::Enum {
            name: ele_name,
            variants,
        } => {
            let make_prefix = |prefix: &Option<LitStr>| {
                if let Some(lit) = prefix {
                    quote!(#lit)
                } else {
                    quote!("")
                }
            };

            let (prefixes, locals): (Vec<_>, Vec<_>) = variants
                .iter()
                .map(|variant| match variant {
                    Fields::Newtype { prefix, tags, .. } => {
                        (vec![make_prefix(prefix); tags.len()], tags.clone())
                    }
                    Fields::Named { prefix, tag, .. } => {
                        (vec![make_prefix(prefix)], vec![tag.clone()])
                    }
                })
                .unzip();

            let read = variants.iter().map(|variant| match variant {
                Fields::Named {
                    tag,
                    name,
                    fields,
                    prefix,
                    namespaces,
                } => named::read(&prefix, &tag, quote!(#ele_name::#name), &fields),
                Fields::Newtype { name, ty, .. } => newtype::read(&ty, quote!(#ele_name::#name)),
            });

            quote! {
                while let Some(tag) = reader.find_element_start(None)? {
                    match tag {
                        #( #( (#prefixes, #locals) )|* => { #read } )*
                        (prefix, local) => {
                            strong_xml::log_skip_element!(#ele_name, prefix, local);
                            // skip the start tag
                            reader.next();
                            reader.read_to_end(prefix, local)?;
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
            } => named::read(&prefix, &tag, quote!(#name), &fields),
            Fields::Newtype { name, ty, .. } => newtype::read(&ty, quote!(#name)),
        },
    }
}
