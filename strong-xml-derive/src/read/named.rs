use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr};

use crate::types::{Field, Type};

pub fn read(tag: &LitStr, ele_name: TokenStream, fields: &[Field]) -> TokenStream {
    let init_fields = fields.iter().map(|field| match field {
        Field::Attribute { bind, ty, .. } => init_value(bind, ty, false),
        Field::Child {
            bind,
            ty,
            container_tag,
            ..
        }
        | Field::FlattenText {
            bind,
            ty,
            container_tag,
            ..
        } => init_value(bind, ty, container_tag.is_some()),
        Field::Text { bind, .. } => quote! { let #bind; },
    });

    let return_fields = fields.iter().map(|field| match field {
        Field::Attribute {
            name,
            bind,
            ty,
            default,
            ..
        } => return_value(name, bind, ty, *default, false, &ele_name),
        Field::Child {
            name,
            bind,
            ty,
            default,
            container_tag,
            ..
        }
        | Field::FlattenText {
            name,
            bind,
            ty,
            default,
            container_tag,
            ..
        } => return_value(name, bind, ty, *default, container_tag.is_some(), &ele_name),
        Field::Text { name, bind, ty, .. } => return_value(name, bind, ty, false, false, &ele_name),
    });

    let read_attr_fields = fields.iter().filter_map(|field| match field {
        Field::Attribute {
            bind,
            ty,
            tag,
            name,
            ..
        } => Some(read_attrs(&tag, &bind, &name, &ty, &ele_name)),
        _ => None,
    });

    let read_child_fields = fields.iter().filter_map(|field| match field {
        Field::Child {
            bind,
            ty,
            tags,
            name,
            container_tag,
            ..
        } => Some(read_children(
            tags,
            bind,
            name,
            ty,
            container_tag.as_ref(),
            &ele_name,
        )),
        _ => None,
    });

    let read_flatten_text_fields = fields.iter().filter_map(|field| match field {
        Field::FlattenText {
            bind,
            ty,
            tag,
            name,
            container_tag,
            ..
        } => Some(read_flatten_text(
            tag,
            bind,
            name,
            ty,
            container_tag.as_ref(),
            &ele_name,
        )),
        _ => None,
    });

    let read_text_fields = fields.iter().filter_map(|field| match field {
        Field::Text { bind, ty, name, .. } => Some(read_text(&tag, bind, name, ty, &ele_name)),
        _ => None,
    });

    let is_text_element = fields
        .iter()
        .any(|field| matches!(field, Field::Text { .. }));

    let return_fields = quote! {
        let __res = #ele_name {
            #( #return_fields, )*
        };

        strong_xml::log_finish_reading!(#ele_name);

        return Ok(__res);
    };

    let read_content = if is_text_element {
        quote! {
            #( #read_text_fields )*
            #return_fields
        }
    } else {
        quote! {
            if let Token::ElementEnd { end: ElementEnd::Empty, .. } = reader.next().unwrap()? {
                #return_fields
            }

            while let Some(__tag) = reader.find_element_start(Some(#tag))? {
                match __tag {
                    #( #read_child_fields, )*
                    #( #read_flatten_text_fields, )*
                    tag => {
                        strong_xml::log_skip_element!(#ele_name, tag);
                        // skip the start tag
                        reader.next();
                        reader.read_to_end(tag)?;
                    },
                }
            }

            #return_fields
        }
    };

    quote! {
        strong_xml::log_start_reading!(#ele_name);

        #( #init_fields )*

        reader.read_till_element_start(#tag)?;

        while let Some((__key, __value)) = reader.find_attribute()? {
            match __key {
                #( #read_attr_fields, )*
                key => {
                    strong_xml::log_skip_attribute!(#ele_name, key);
                },
            }
        }

        #read_content
    }
}

fn init_value(name: &Ident, ty: &Type, is_container: bool) -> TokenStream {
    if ty.is_vec() && !is_container {
        quote! { let mut #name = Vec::new(); }
    } else {
        quote! { let mut #name = None; }
    }
}

fn return_value(
    name: &TokenStream,
    bind: &Ident,
    ty: &Type,
    default: bool,
    is_container: bool,
    ele_name: &TokenStream,
) -> TokenStream {
    if (ty.is_vec() && !is_container) || ty.is_option() {
        quote! { #name: #bind }
    } else if default {
        quote! { #name: #bind.unwrap_or_default() }
    } else {
        quote! {
            #name: #bind.ok_or(XmlError::MissingField {
                name: stringify!(#ele_name).to_owned(),
                field: stringify!(#name).to_owned(),
            })?
        }
    }
}

fn read_attrs(
    tag: &LitStr,
    bind: &Ident,
    name: &TokenStream,
    ty: &Type,
    ele_name: &TokenStream,
) -> TokenStream {
    let from_str = from_str(ty);

    if ty.is_vec() {
        panic!("`attr` attribute doesn't support Vec.");
    } else {
        quote! {
            #tag => {
                strong_xml::log_start_reading_field!(#ele_name, #name);

                #bind = Some(#from_str);

                strong_xml::log_finish_reading_field!(#ele_name, #name);
            }
        }
    }
}

fn read_text(
    tag: &LitStr,
    bind: &Ident,
    name: &TokenStream,
    ty: &Type,
    ele_name: &TokenStream,
) -> TokenStream {
    let from_str = from_str(ty);

    if ty.is_vec() {
        panic!("`text` attribute doesn't support Vec.");
    } else {
        quote! {
            strong_xml::log_start_reading_field!(#ele_name, #name);

            let __value = reader.read_text(#tag)?;
            #bind = Some(#from_str);

            strong_xml::log_finish_reading_field!(#ele_name, #name);
        }
    }
}

fn read_children(
    tags: &[LitStr],
    bind: &Ident,
    name: &TokenStream,
    ty: &Type,
    container_tag: Option<&LitStr>,
    ele_name: &TokenStream,
) -> TokenStream {
    let match_tags = match container_tag {
        Some(container_tag) => quote! { #container_tag },
        None => quote! { #( #tags )|* },
    };

    let from_reader = if let Some(container_tag) = container_tag {
        let inner_reader = match &ty {
            Type::VecT(ty) | Type::OptionVecT(ty) => quote! {
                #bind.as_mut().unwrap().push(<#ty as strong_xml::XmlRead>::from_reader(reader)?);
            },
            _ => panic!("`child` + `container` attribute only supports Vec<T> and Option<Vec<T>>."),
        };

        quote! {
            reader.next().unwrap()?;

            if #bind.is_some() {
                return Err(XmlError::DuplicateField {
                    field: #container_tag.to_owned(),
                });
            }

            while let Some(_) = reader.find_attribute()? {};

            #bind = Some(Vec::new());
            if let Token::ElementEnd { end: ElementEnd::Open, .. } = reader.next().unwrap()? {
                while let Some(__tag) = reader.find_element_start(Some(#container_tag))? {
                    match __tag {
                        #( #tags )|* => {
                            strong_xml::log_start_reading_field!(#ele_name, #name);
                            #inner_reader
                            strong_xml::log_finish_reading_field!(#ele_name, #name);
                        }
                        tag => {
                            strong_xml::log_skip_element!(#ele_name, tag);
                            // skip the start tag
                            reader.next();
                            reader.read_to_end(tag)?;
                        },
                    }
                }
            }

        }
    } else {
        match &ty {
            Type::VecT(ty) => quote! {
                #bind.push(<#ty as strong_xml::XmlRead>::from_reader(reader)?);
            },
            Type::OptionT(ty) | Type::T(ty) => quote! {
                if #bind.is_some() {
                    return Err(XmlError::DuplicateField {
                        field: stringify!(#( #tags )|*).to_owned(),
                    });
                }
                #bind = Some(<#ty as strong_xml::XmlRead>::from_reader(reader)?);
            },
            _ => panic!("`child` attribute only supports Vec<T>, Option<T> and T."),
        }
    };

    quote! {
        #match_tags => {
            strong_xml::log_start_reading_field!(#ele_name, #name);

            #from_reader

            strong_xml::log_finish_reading_field!(#ele_name, #name);
        }
    }
}

fn read_flatten_text(
    tag: &LitStr,
    bind: &Ident,
    name: &TokenStream,
    ty: &Type,
    container_tag: Option<&LitStr>,
    ele_name: &TokenStream,
) -> TokenStream {
    let match_tag = container_tag.unwrap_or(tag);

    let from_str = from_str(ty);

    let read_text = if let Some(container_tag) = container_tag {
        quote! {
            if #bind.is_some() {
                return Err(XmlError::DuplicateField {
                    field: #container_tag.to_owned(),
                });
            }

            while let Some(_) = reader.find_attribute()? {};

            #bind = Some(Vec::new());
            if let Token::ElementEnd { end: ElementEnd::Open, .. } = reader.next().unwrap()? {
                while let Some(__tag) = reader.find_element_start(Some(#container_tag))? {
                    match __tag {
                        #tag => {
                            strong_xml::log_start_reading_field!(#ele_name, #name);
                            reader.next().unwrap()?;
                            let __value = reader.read_text(#tag)?;
                            #bind.as_mut().unwrap().push(#from_str);
                            strong_xml::log_finish_reading_field!(#ele_name, #name);
                        }
                        tag => {
                            strong_xml::log_skip_element!(#ele_name, tag);
                            // skip the start tag
                            reader.next();
                            reader.read_to_end(tag)?;
                        },
                    }
                }
            }

        }
    } else if ty.is_vec() {
        quote! {
            let __value = reader.read_text(#tag)?;
            #bind.push(#from_str);
        }
    } else {
        quote! {
            if #bind.is_some() {
                return Err(XmlError::DuplicateField {
                    field: #tag.to_owned(),
                });
            }
            let __value = reader.read_text(#tag)?;
            #bind = Some(#from_str);
        }
    };

    quote! {
        #match_tag => {
            // skip element start
            reader.next();

            strong_xml::log_start_reading_field!(#ele_name, #name);

            #read_text

            strong_xml::log_finish_reading_field!(#ele_name, #name);
        }
    }
}

fn from_str(ty: &Type) -> TokenStream {
    match &ty {
        Type::CowStr | Type::OptionCowStr | Type::VecCowStr | Type::OptionVecCowStr => {
            quote! { __value }
        }
        Type::Bool | Type::OptionBool | Type::VecBool | Type::OptionVecBool => quote! {
            match &*__value {
                "t" | "true" | "y" | "yes" | "on" | "1" => true,
                "f" | "false" | "n" | "no" | "off" | "0" => false,
                _ => <bool as std::str::FromStr>::from_str(&__value).map_err(|e| XmlError::FromStr(e.into()))?
            }
        },
        Type::T(ty) | Type::OptionT(ty) | Type::VecT(ty) | Type::OptionVecT(ty) => quote! {
            <#ty as std::str::FromStr>::from_str(&__value).map_err(|e| XmlError::FromStr(e.into()))?
        },
    }
}
