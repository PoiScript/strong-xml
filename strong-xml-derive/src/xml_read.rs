use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
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

    let init_fields = attr_fields
        .iter()
        .map(|e| (&e.name, &e.ty))
        .chain(text_field.iter().map(|e| (&e.name, &e.ty)))
        .chain(child_fields.iter().map(|e| (&e.name, &e.ty)))
        .chain(flatten_text_fields.iter().map(|e| (&e.name, &e.ty)))
        .map(|(name, ty)| init_value(name, ty));

    let return_fields = attr_fields
        .iter()
        .map(|e| (&e.name, &e.ty, e.default))
        .chain(text_field.iter().map(|e| (&e.name, &e.ty, true)))
        .chain(child_fields.iter().map(|e| (&e.name, &e.ty, e.default)))
        .chain(
            flatten_text_fields
                .iter()
                .map(|e| (&e.name, &e.ty, e.default)),
        )
        .map(|(name, ty, default)| return_value(name, ty, default, ele_name));

    let read_attr_fields = attr_fields
        .iter()
        .map(|e| read_attrs(&e.tag, &e.name, &e.ty, ele_name));

    let has_text_field = text_field.is_some();

    let read_text_field = text_field
        .as_ref()
        .map(|e| read_text(tag, &e.name, &e.ty, ele_name));

    let read_child_fields = child_fields
        .iter()
        .map(|e| read_children(&e.tags, &e.name, &e.ty, ele_name));

    let read_flatten_text_fields = flatten_text_fields
        .iter()
        .map(|e| read_flatten_text(&e.tag, &e.name, &e.ty, ele_name));

    let return_fields = quote! {
        #( #return_fields, )*
    };

    quote! {
        log::debug!(concat!("[", stringify!(#ele_name), "] Started reading"));

        #( #init_fields )*

        reader.read_till_element_start(#tag)?;

        while let Some(__token) = reader.next() {
            match __token? {
                Token::Attribute { span: __span, value: __value, .. } => {
                    let __value = __value.as_str();
                    let __span = __span.as_str();
                    let __key = &__span[0..__span.len() - __value.len() - 3];
                    let __value = std::borrow::Cow::Borrowed(__value);
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
    if ty.is_vec() {
        quote! { let mut #name = Vec::new(); }
    } else {
        quote! { let mut #name = None; }
    }
}

fn return_value(name: &Ident, ty: &Type, default: bool, ele_name: &Ident) -> TokenStream {
    if ty.is_vec() || ty.is_option() {
        quote! { #name }
    } else if default {
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

fn read_attrs(tag: &LitStr, name: &Ident, ty: &Type, ele_name: &Ident) -> TokenStream {
    let from_str = from_str(ty);

    if ty.is_vec() {
        panic!("`attr` attribute doesn't support Vec.");
    } else {
        quote! {
            #tag => {
                log::trace!(
                    concat!("[", stringify!(#ele_name), "] Reading attribute field `", stringify!(#name), "`")
                );

                #name = Some(#from_str);
            }
        }
    }
}

fn read_text(tag: &LitStr, name: &Ident, ty: &Type, ele_name: &Ident) -> TokenStream {
    let from_str = from_str(ty);

    if ty.is_vec() {
        panic!("`text` attribute doesn't support Vec.");
    } else {
        quote! {
            log::trace!(
                concat!("[", stringify!(#ele_name), "] Reading text field `", stringify!(#name), "`")
            );

            let __value = reader.read_text(#tag)?;
            #name = Some(#from_str);
        }
    }
}

fn read_children(tags: &[LitStr], name: &Ident, ty: &Type, ele_name: &Ident) -> TokenStream {
    let tags = tags.iter();

    match &ty {
        Type::VecT(ty) => {
            if let Some(ident) = trim_lifetime(ty) {
                quote! {
                    #( #tags )|* => {
                        log::trace!(
                            concat!("[", stringify!(#ele_name), "] Reading children field `", stringify!(#name), "`")
                        );

                        #name.push(#ident::from_reader(reader)?)
                    }
                }
            } else {
                quote! {
                    #( #tags )|* => {
                        log::trace!(
                            concat!("[", stringify!(#ele_name), "] Reading children field `", stringify!(#name), "`")
                        );

                        #name.push(#ty::from_reader(reader)?);
                    }
                }
            }
        }
        Type::OptionT(ty) | Type::T(ty) => {
            if let Some(ident) = trim_lifetime(ty) {
                quote! {
                    #( #tags )|* => {
                        log::trace!(
                            concat!("[", stringify!(#ele_name), "] Reading children field `", stringify!(#name), "`")
                        );

                        #name = Some(#ident::from_reader(reader)?);
                    }
                }
            } else {
                quote! {
                    #( #tags )|* => {
                        log::trace!(
                            concat!("[", stringify!(#ele_name), "] Reading children field `", stringify!(#name), "`")
                        );
                        #name = Some(#ty::from_reader(reader)?);
                    }
                }
            }
        }
        _ => panic!("`child` attribute only supports Vec<T>, Option<T> and T."),
    }
}

fn read_flatten_text(tag: &LitStr, name: &Ident, ty: &Type, ele_name: &Ident) -> TokenStream {
    let from_str = from_str(ty);

    if ty.is_vec() {
        quote! {
            #tag => {
                // skip element start
                reader.next();

                log::trace!(
                    concat!("[", stringify!(#ele_name), "] Reading flatten_text field `", stringify!(#name), "`")
                );

                let __value = reader.read_text(#tag)?;
                #name.push(#from_str);
            }
        }
    } else {
        quote! {
            #tag => {
                // skip element start
                reader.next();

                log::trace!(
                    concat!("[", stringify!(#ele_name), "] Reading flatten_text field `", stringify!(#name), "`")
                );

                let __value = reader.read_text(#tag)?;
                #name = Some(#from_str);
            }
        }
    }
}

fn from_str(ty: &Type) -> TokenStream {
    match &ty {
        Type::CowStr | Type::OptionCowStr | Type::VecCowStr => quote! { __value },
        Type::Bool | Type::OptionBool | Type::VecBool => quote! {
            match &*__value {
                "t" | "true" | "y" | "yes" | "on" | "1" => true,
                "f" | "false" | "n" | "no" | "off" | "0" => false,
                _ => <bool as std::str::FromStr>::from_str(&__value).map_err(|e| XmlError::FromStr(e.into()))?
            }
        },
        Type::T(ty) | Type::OptionT(ty) | Type::VecT(ty) => {
            if let Some(ty) = trim_lifetime(ty) {
                quote! { <#ty as std::str::FromStr>::from_str(&__value).map_err(|e| XmlError::FromStr(e.into()))? }
            } else {
                quote! { <#ty as std::str::FromStr>::from_str(&__value).map_err(|e| XmlError::FromStr(e.into()))? }
            }
        }
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
