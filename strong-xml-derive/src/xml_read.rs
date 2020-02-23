use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr};

use crate::types::{
    trim_lifetime, Element, EnumElement, Field, LeafElement, ParentElement, TextElement, Type,
};

pub fn read(element: &Element) -> TokenStream {
    match element {
        Element::Enum(enum_ele) => read_enum_element(enum_ele),
        Element::Leaf(leaf_ele) => read_leaf_element(leaf_ele),
        Element::Text(text_ele) => read_text_element(text_ele),
        Element::Parent(parent_ele) => read_parent_element(parent_ele),
    }
}

fn read_leaf_element(leaf_ele: &LeafElement) -> TokenStream {
    let ele_name = &leaf_ele.name;
    let tag = &leaf_ele.tag;
    let attrs = &leaf_ele.attributes;

    let init_attrs = attrs.iter().map(|e| init_value(&e.1));
    let read_attrs = attrs.iter().map(|e| read_attrs(&e.0, &e.1));
    let return_attrs = attrs.iter().map(|e| return_value(&e.1, ele_name));

    quote! {
        log::debug!("Started reading LeafElement {}.", stringify!(#ele_name));

        #( #init_attrs )*

        strong_xml::utils::read_till_element_start(&mut reader, #tag)?;

        while let Some(__token) = reader.next() {
            match __token? {
                Token::Attribute { span: __span, value: __value, .. } => {
                    let __value = __value.as_str();
                    let __span = &__span.as_str();
                    let __key = &__span[0..__span.len() - __value.len() - 3];
                    match __key {
                        #( #read_attrs, )*
                        _ => log::info!(
                            "Unhandled attribute: {:?} when reading LeafElement {}. Skipping.",
                            __key, stringify!(#ele_name)
                        ),
                    }
                }
                Token::ElementEnd { end: ElementEnd::Empty, .. } => {
                    let __res = #ele_name {
                        #( #return_attrs, )*
                    };

                    log::debug!("Finished reading LeafElement {}.", stringify!(#ele_name));

                    return Ok(__res);
                },
                __token => {
                    return Err(XmlError::UnexpectedToken{ token: format!("{:?}", __token) });
                }
            }
        }

        Err(XmlError::UnexpectedEof)
    }
}

fn read_text_element(text_ele: &TextElement) -> TokenStream {
    let ele_name = &text_ele.name;
    let tag = &text_ele.tag;
    let text = &text_ele.text.name;
    let attrs = &text_ele.attributes;

    let init_attrs = attrs.iter().map(|e| init_value(&e.1));
    let read_attrs = attrs.iter().map(|e| read_attrs(&e.0, &e.1));
    let return_attrs = attrs.iter().map(|e| return_value(&e.1, ele_name));

    quote! {
        log::debug!("Started reading TextElement {}.", stringify!(#ele_name));

        #( #init_attrs )*

        strong_xml::utils::read_till_element_start(&mut reader, #tag)?;

        while let Some(token) = reader.next() {
            match token? {
                Token::Attribute { span: __span, value: __value, .. } => {
                    let __value = __value.as_str();
                    let __span = __span.as_str();
                    let __key = &__span[0..__span.len() - __value.len() - 3];
                    match __key {
                        #( #read_attrs, )*
                        _ => log::info!(
                            "Unhandled attribute: {:?} when reading TextElement {}. Skipping.",
                            __key, stringify!(#ele_name)
                        ),
                    }
                }
                Token::ElementEnd { end: ElementEnd::Open, .. } => {
                    let #text = strong_xml::utils::read_text(&mut reader, #tag)?;

                    let __res = #ele_name {
                        #text,
                        #( #return_attrs, )*
                    };

                    log::debug!("Finished reading TextElement {}.", stringify!(#ele_name));

                    return Ok(__res);
                },
                __token => {
                    return Err(XmlError::UnexpectedToken{ token: format!("{:?}", __token) });
                }
            }
        }
        Err(XmlError::UnexpectedEof)
    }
}

fn read_parent_element(parent_ele: &ParentElement) -> TokenStream {
    let ele_name = &parent_ele.name;
    let tag = &parent_ele.tag;
    let attrs = &parent_ele.attributes;
    let children = &parent_ele.children;
    let flatten_text = &parent_ele.flatten_text;

    let mut children_fields = children.iter().map(|e| &e.1).collect::<Vec<_>>();
    children_fields.dedup_by_key(|f| &f.name);

    let init_attrs = attrs.iter().map(|e| init_value(&e.1));
    let read_attrs = attrs.iter().map(|e| read_attrs(&e.0, &e.1));
    let return_attrs = attrs.iter().map(|e| return_value(&e.1, ele_name));

    let init_children = children_fields.iter().map(|f| init_value(f));
    let read_children = children.iter().map(|e| read_children(&e.0, &e.1));
    let return_children = children_fields.iter().map(|f| return_value(f, ele_name));

    let init_flatten_text = flatten_text.iter().map(|e| init_value(&e.1));
    let read_flatten_text = flatten_text.iter().map(|e| read_flatten_text(&e.0, &e.1));
    let return_flatten_text = flatten_text.iter().map(|e| return_value(&e.1, ele_name));

    let return_fields = quote! {
        #( #return_attrs, )*
        #( #return_children, )*
        #( #return_flatten_text, )*
    };

    quote! {
        log::debug!("Started reading ParentElement {}.", stringify!(#ele_name));

        #( #init_attrs )*
        #( #init_children )*
        #( #init_flatten_text )*

        strong_xml::utils::read_till_element_start(&mut reader, #tag)?;

        while let Some(__token) = reader.next() {
            match __token? {
                Token::Attribute { span: __span, value: __value, .. } => {
                    let __value = __value.as_str();
                    let __span = __span.as_str();
                    let __key = &__span[0..__span.len() - __value.len() - 3];
                    match __key {
                        #( #read_attrs, )*
                        _ => log::info!(
                            "Unhandled attribute: {:?} when parsing ParentElement {}. Skipping.",
                            __key, stringify!(#ele_name)
                        ),
                    }
                }
                Token::ElementEnd { end: ElementEnd::Open, .. } => {
                    while let Some(__token) = reader.peek() {
                        match __token {
                            Ok(Token::ElementStart { span: __span, .. }) => {
                                match &__span.as_str()[1..] {
                                    #( #read_children, )*
                                    #( #read_flatten_text, )*
                                    __tag => {
                                        log::info!(
                                            "Unhandled tag: {:?} when parsing ParentElement {}. Skipping.",
                                            __tag, stringify!(#ele_name)
                                        );
                                        // skip the start tag
                                        reader.next();
                                        strong_xml::utils::read_to_end(reader, __tag)?;
                                    },
                                }
                            }
                            Ok(Token::ElementEnd { end: ElementEnd::Close(_, _), span: __span }) => {
                                let __span = __span.as_str();
                                let __tag = &__span[2..__span.len() - 1];
                                if __tag == #tag {
                                    let __res = #ele_name { #return_fields };

                                    log::debug!("Finished reading ParentElmenet {}.", stringify!(#ele_name));

                                    reader.next();

                                    return Ok(__res);
                                } else {
                                    return Err(XmlError::TagMismatch {
                                        expected: #tag.to_owned(),
                                        found: __tag.to_owned(),
                                    });
                                }
                            }
                            Ok(Token::ElementEnd { .. }) |
                            Ok(Token::Attribute { .. }) |
                            Ok(Token::Text { .. }) |
                            Ok(Token::Cdata { .. }) => {
                                return Err(XmlError::UnexpectedToken{ token: format!("{:?}", __token) });
                            }
                            _ => (),
                        }
                    }
                    return Err(XmlError::UnexpectedEof);
                }
                Token::ElementEnd { end: ElementEnd::Empty, .. } => {
                    let __res = #ele_name { #return_fields };

                    log::debug!("Finished reading ParentElmenet {}.", stringify!(#ele_name));

                    return Ok(__res);
                }
                __token => {
                    return Err(XmlError::UnexpectedToken{ token: format!("{:?}", __token) });
                }
            }
        }
        Err(XmlError::UnexpectedEof)
    }
}

fn init_value(field: &Field) -> TokenStream {
    let name = &field.name;

    match field.ty {
        Type::VecT(_) | Type::VecCowStr => quote! { let mut #name = vec![]; },
        Type::OptionCowStr
        | Type::OptionT(_)
        | Type::OptionBool
        | Type::OptionUsize
        | Type::CowStr
        | Type::T(_)
        | Type::Bool
        | Type::Usize => quote! { let mut #name = None; },
    }
}

fn return_value(field: &Field, ele_name: &Ident) -> TokenStream {
    let name = &field.name;

    match field.ty {
        Type::OptionCowStr
        | Type::OptionT(_)
        | Type::OptionBool
        | Type::OptionUsize
        | Type::VecCowStr
        | Type::VecT(_) => quote! { #name },
        Type::CowStr | Type::T(_) | Type::Usize | Type::Bool => {
            quote! {
                #name: #name.ok_or(XmlError::MissingField {
                    name: stringify!(#ele_name).to_owned(),
                    field: stringify!(#name).to_owned(),
                })?
            }
        }
    }
}

fn read_attrs(tag: &LitStr, field: &Field) -> TokenStream {
    let name = &field.name;

    match &field.ty {
        Type::CowStr | Type::OptionCowStr => quote! {
            #tag => #name = Some(Cow::Borrowed(__value))
        },
        Type::Bool | Type::OptionBool => quote! {
            #tag => {
                use std::str::FromStr;
                #name = Some(bool::from_str(__value).or(usize::from_str(__value).map(|v| v != 0))?);
            }
        },
        Type::Usize | Type::OptionUsize => quote! {
            #tag => {
                use std::str::FromStr;
                #name = Some(usize::from_str(__value)?);
            }
        },
        Type::T(ty) | Type::OptionT(ty) => quote! {
            #tag => {
                use std::str::FromStr;
                #name = Some(#ty::from_str(__value)?);
            }
        },
        _ => panic!("#[xml(attr =\"\")] only supports Cow<str>, Option<Cow<str>>, bool, Option<bool>, usize, Option<usize> and Option<T>.")
    }
}

fn read_children(tag: &LitStr, field: &Field) -> TokenStream {
    let name = &field.name;

    match &field.ty {
        Type::VecT(ty) => {
            if let Some(ident) = trim_lifetime(ty) {
                quote! {
                    #tag => #name.push(#ident::from_reader(reader)?)
                }
            } else {
                quote! {
                    #tag => #name.push(#ty::from_reader(reader)?)
                }
            }
        }
        Type::OptionT(ty) => {
            if let Some(ident) = trim_lifetime(ty) {
                quote! {
                    #tag => #name = Some(#ident::from_reader(reader)?)
                }
            } else {
                quote! {
                    #tag => #name = Some(#ty::from_reader(reader)?)
                }
            }
        }
        Type::T(ty) => {
            if let Some(ident) = trim_lifetime(ty) {
                quote! {
                    #tag => #name = Some(#ident::from_reader(reader)?)
                }
            } else {
                quote! {
                    #tag => #name = Some(#ty::from_reader(reader)?)
                }
            }
        }
        _ => panic!("#[xml(child = \"\")] only support Vec<T>, Option<T> and T."),
    }
}

fn read_flatten_text(tag: &LitStr, field: &Field) -> TokenStream {
    let name = &field.name;

    match field.ty {
        Type::VecCowStr => quote! {
            #tag => {
                // skip element start
                reader.next();
                log::debug!("Started reading flatten_text {}.", stringify!(#name));
                #name.push(strong_xml::utils::read_text(&mut reader, #tag)?);
                log::debug!("Finished reading flatten_text {}.", stringify!(#name));
            }
        },
        Type::CowStr | Type::OptionCowStr => quote! {
            #tag => {
                // skip element start
                reader.next();
                log::debug!("Started reading flatten_text {}.", stringify!(#name));
                #name = Some(strong_xml::utils::read_text(&mut reader, #tag)?);
                log::debug!("Finished reading flatten_text {}.", stringify!(#name));
            }
        },
        _ => panic!(
            "#[xml(flatten_text)] only support Cow<str>, Vec<Cow<str>> and Option<Cow<str>>."
        ),
    }
}

fn read_enum_element(enum_ele: &EnumElement) -> TokenStream {
    let ele_name = &enum_ele.name;
    let read_variants = enum_ele.elements.iter().map(|(tag, var)| {
        let var_name = &var.name;
        let ty = &var.ty;

        if let Some(ident) = trim_lifetime(ty) {
            quote! {
                #tag => return #ident::from_reader(reader).map(#ele_name::#var_name)
            }
        } else {
            quote! {
                #tag => return #ty::from_reader(reader).map(#ele_name::#var_name)
            }
        }
    });

    quote! {
        while let Some(__token) = reader.peek() {
            match __token {
                Ok(Token::ElementStart { span: __span, .. }) => {
                    let __tag = &__span.as_str()[1..];
                    match &__span.as_str()[1..] {
                        #( #read_variants, )*
                        __tag => {
                            log::info!(
                                "Unhandled tag: {:?} when parsing {}. Skipping.",
                                __tag, stringify!(#ele_name)
                            );
                            // skip the start tag
                            reader.next();
                            strong_xml::utils::read_to_end(reader, __tag)?;
                        }
                    }
                },
                Ok(Token::ElementEnd { .. }) |
                Ok(Token::Attribute { .. }) |
                Ok(Token::Text { .. }) |
                Ok(Token::Cdata { .. }) => {
                    return Err(XmlError::UnexpectedToken{ token: format!("{:?}", __token) });
                },
                _ => (),
            }
        }
        Err(XmlError::UnexpectedEof)
    }
}
