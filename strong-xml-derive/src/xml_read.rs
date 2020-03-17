use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::{Ident, LitStr};

use crate::types::{trim_lifetime, Element, EnumElement, StructElement, Type};

pub fn read(element: &Element) -> TokenStream {
    match element {
        Element::Enum(enum_ele) => read_enum_element(enum_ele),
        Element::Struct(struct_ele) => read_struct_element(struct_ele),
    }
}

fn read_struct_element(struct_ele: &StructElement) -> TokenStream {
    let ele_name = &struct_ele.name;
    let tag = &struct_ele.tag;
    let attr_fields = &struct_ele.attributes;
    let child_fields = &struct_ele.children;
    let flatten_text_fields = &struct_ele.flatten_text;
    let text_field = &struct_ele.text;

    let init_attr_fields = attr_fields.iter().map(|e| init_value(&e.name, &e.ty));
    let read_attr_fields = attr_fields
        .iter()
        .map(|e| read_attrs(&e.tag, &e.name, &e.ty));
    let return_attr_fields = attr_fields
        .iter()
        .map(|e| return_value(&e.name, &e.ty, e.default, ele_name));

    let has_text_field = text_field.is_some();
    let init_text_field = text_field.iter().map(|e| init_value(&e.name, &e.ty));
    let read_text_field = text_field.as_ref().map(|e| {
        let name = &e.name;
        quote! {
            #name = Some(reader.read_text(#tag)?);
        }
    });
    let return_text_field = text_field
        .iter()
        .map(|e| return_value(&e.name, &e.ty, true, ele_name));

    let init_child_fields = child_fields.iter().map(|e| init_value(&e.name, &e.ty));
    let read_child_fields = child_fields
        .iter()
        .map(|e| read_children(&e.tags, &e.name, &e.ty));
    let return_child_fields = child_fields
        .iter()
        .map(|e| return_value(&e.name, &e.ty, e.default, ele_name));

    let init_flatten_text_fields = flatten_text_fields
        .iter()
        .map(|e| init_value(&e.name, &e.ty));
    let read_flatten_text_fields = flatten_text_fields
        .iter()
        .map(|e| read_flatten_text(&e.tag, &e.name, &e.ty));
    let return_flatten_text_fields = flatten_text_fields
        .iter()
        .map(|e| return_value(&e.name, &e.ty, e.default, ele_name));

    let return_fields = quote! {
        #( #return_attr_fields, )*
        #( #return_child_fields, )*
        #( #return_flatten_text_fields, )*
        #( #return_text_field, )*
    };

    quote! {
        log::debug!(concat!("[", stringify!(#ele_name), "] Started reading"));

        #( #init_attr_fields )*
        #( #init_child_fields )*
        #( #init_flatten_text_fields )*
        #( #init_text_field )*

        reader.read_till_element_start(#tag)?;

        while let Some(__token) = reader.next() {
            match __token? {
                Token::Attribute { span: __span, value: __value, .. } => {
                    let __value = __value.as_str();
                    let __span = __span.as_str();
                    let __key = &__span[0..__span.len() - __value.len() - 3];
                    match __key {
                        #( #read_attr_fields, )*
                        key => log::info!(
                            concat!("[", stringify!(#ele_name), "] Skip attribute `{}`"),
                            key
                        ),
                    }
                }
                Token::ElementEnd { end: ElementEnd::Open, .. } => {
                    break;
                }
                Token::ElementEnd { end: ElementEnd::Empty, .. } => {
                    let __res = #ele_name { #return_fields };

                    log::debug!(
                        concat!("[", stringify!(#ele_name), "] Finished reading")
                    );

                    return Ok(__res);
                }
                __token => {
                    return Err(XmlError::UnexpectedToken {
                        token: format!("{:?}", __token),
                    });
                }
            }
        }

        if #has_text_field {
            #read_text_field
            let __res = #ele_name { #return_fields };

            log::debug!(
                concat!("[", stringify!(#ele_name), "] Finished reading")
            );

            return Ok(__res);
        }

        while let Some(__token) = reader.peek() {
            match __token.as_ref() {
                Ok(Token::ElementStart { span: __span, .. }) => {
                    match &__span.as_str()[1..] {
                        #( #read_child_fields, )*
                        #( #read_flatten_text_fields, )*
                        __tag => {
                            log::info!(
                                concat!("[", stringify!(#ele_name), "] Skip element `{}`"),
                                __tag
                            );
                            // skip the start tag
                            reader.next();
                            reader.read_to_end(__tag)?;
                        },
                    }
                }
                Ok(Token::ElementEnd { end: ElementEnd::Close(_, _), span: __span }) => {
                    let __span = __span.as_str();
                    let __tag = &__span[2..__span.len() - 1];
                    if __tag == #tag {
                        let __res = #ele_name { #return_fields };

                        log::debug!(
                            concat!("[", stringify!(#ele_name), "] Finished reading")
                        );

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
                    return Err(XmlError::UnexpectedToken {
                        token: format!("{:?}", __token),
                    });
                }
                _ => (),
            }
        }
        Err(XmlError::UnexpectedEof)
    }
}

fn init_value(name: &Ident, ty: &Type) -> TokenStream {
    match ty {
        Type::VecT(_) | Type::VecCowStr => quote! { let mut #name = Vec::new(); },
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

fn return_value(name: &Ident, ty: &Type, default: bool, ele_name: &Ident) -> TokenStream {
    match ty {
        Type::OptionCowStr
        | Type::OptionT(_)
        | Type::OptionBool
        | Type::OptionUsize
        | Type::VecCowStr
        | Type::VecT(_) => quote! { #name },
        Type::CowStr | Type::T(_) | Type::Usize | Type::Bool => {
            if default {
                quote! { #name: #name.unwrap_or_default() }
            } else {
                quote! {
                    #name: #name.ok_or(XmlError::MissingField {
                        name: stringify!(#ele_name).to_owned(),
                        field: stringify!(#name).to_owned(),
                    })?
                }
            }
        }
    }
}

fn read_attrs(tag: &LitStr, name: &Ident, ty: &Type) -> TokenStream {
    match &ty {
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

fn read_children(tags: &[LitStr], name: &Ident, ty: &Type) -> TokenStream {
    let tags = tags.iter();

    match &ty {
        Type::VecT(ty) => {
            if let Some(ident) = trim_lifetime(ty) {
                quote! {
                    #( #tags )|* => #name.push(#ident::from_reader(reader)?)
                }
            } else {
                quote! {
                    #( #tags )|* => #name.push(#ty::from_reader(reader)?)
                }
            }
        }
        Type::OptionT(ty) | Type::T(ty) => {
            if let Some(ident) = trim_lifetime(ty) {
                quote! {
                    #( #tags )|* => #name = Some(#ident::from_reader(reader)?)
                }
            } else {
                quote! {
                    #( #tags )|* => #name = Some(#ty::from_reader(reader)?)
                }
            }
        }
        _ => panic!("#[xml(child = \"\")] only support Vec<T>, Option<T> and T."),
    }
}

fn read_flatten_text(tag: &LitStr, name: &Ident, ty: &Type) -> TokenStream {
    match ty {
        Type::VecCowStr => quote! {
            #tag => {
                // skip element start
                reader.next();
                log::debug!("Started reading flatten_text {}.", stringify!(#name));
                #name.push(reader.read_text(#tag)?);
                log::debug!("Finished reading flatten_text {}.", stringify!(#name));
            }
        },
        Type::CowStr | Type::OptionCowStr => quote! {
            #tag => {
                // skip element start
                reader.next();
                log::debug!("Started reading flatten_text {}.", stringify!(#name));
                #name = Some(reader.read_text(#tag)?);
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
    let tags = enum_ele.elements.iter().map(|var| &var.tags);
    let name = enum_ele.elements.iter().map(|var| &var.name);
    let ty = enum_ele.elements.iter().map(|var| {
        if let Some(ty) = trim_lifetime(&var.ty) {
            ty.to_token_stream()
        } else {
            var.ty.to_token_stream()
        }
    });

    quote! {
        while let Some(token) = reader.peek() {
            match token {
                Ok(Token::ElementStart { span, .. }) => {
                    match &span.as_str()[1..] {
                        #(
                            #( #tags )|* => return #ty::from_reader(reader).map(#ele_name::#name),
                        )*
                        tag => {
                            log::info!(
                                concat!("[", stringify!(#ele_name), "] Skip element {}"),
                                tag
                            );
                            // skip the start tag
                            reader.next();
                            reader.read_to_end(tag)?;
                        }
                    }
                },
                Ok(Token::ElementEnd { .. }) |
                Ok(Token::Attribute { .. }) |
                Ok(Token::Text { .. }) |
                Ok(Token::Cdata { .. }) => {
                    return Err(XmlError::UnexpectedToken{ token: format!("{:?}", token) });
                },
                _ => (),
            }
        }
        Err(XmlError::UnexpectedEof)
    }
}
