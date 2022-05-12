use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr};

use crate::types::{Field, Type, QName};

pub fn read(
    tag: &QName,
    ele_name: TokenStream,
    fields: &[Field],
) -> TokenStream {
    let init_fields = fields.iter().map(|field| match field {
        Field::Attribute { bind, ty, .. }
        | Field::Child { bind, ty, .. }
        | Field::FlattenText { bind, ty, .. } => init_value(bind, ty),
        Field::Text { bind, .. } => quote! { let #bind; },
    });

    let return_fields = fields.iter().map(|field| match field {
        Field::Attribute {
            name,
            bind,
            ty,
            default,
            ..
        }
        | Field::Child {
            name,
            bind,
            ty,
            default,
            ..
        }
        | Field::FlattenText {
            name,
            bind,
            ty,
            default,
            ..
        } => return_value(name, bind, ty, *default, &ele_name),
        Field::Text { name, bind, ty, .. } => return_value(name, bind, ty, false, &ele_name),
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
            ..
        } => Some(read_children(&tags, bind, name, ty, &ele_name)),
        _ => None,
    });

    let read_flatten_text_fields = fields.iter().filter_map(|field| match field {
        Field::FlattenText {
            bind,
            ty,
            tag,
            name,
            ..
        } => Some(read_flatten_text(tag, bind, name, ty, &ele_name)),
        _ => None,
    });

    let read_text_fields = fields.iter().filter_map(|field| match field {
        Field::Text { bind, ty, name, .. } => {
            Some(read_text(&tag, bind, name, ty, &ele_name))
        }
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

            while let Some((__name)) = reader.find_element_start(Some(#tag))? {
                match __name {
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

fn init_value(name: &Ident, ty: &Type) -> TokenStream {
    if ty.is_vec() {
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
    ele_name: &TokenStream,
) -> TokenStream {
    if ty.is_vec() || ty.is_option() {
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
    tag: &QName,
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
    tag: &QName,
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
    tags: &[QName],
    bind: &Ident,
    name: &TokenStream,
    ty: &Type,
    ele_name: &TokenStream,
) -> TokenStream {
    let from_reader = match &ty {
        Type::VecT(ty) => quote! {
            #bind.push(<#ty as strong_xml::XmlRead>::from_reader(reader)?);
        },
        Type::OptionT(ty) | Type::T(ty) => quote! {
            #bind = Some(<#ty as strong_xml::XmlRead>::from_reader(reader)?);
        },
        _ => panic!("`child` attribute only supports Vec<T>, Option<T> and T."),
    };

    quote! {
        #( #tags )|* => {
            strong_xml::log_start_reading_field!(#ele_name, #name);

            #from_reader

            strong_xml::log_finish_reading_field!(#ele_name, #name);
        }
    }
}

fn read_flatten_text(
    tag: &QName,
    bind: &Ident,
    name: &TokenStream,
    ty: &Type,
    ele_name: &TokenStream,
) -> TokenStream {
    let from_str = from_str(ty);

    let read_text = if ty.is_vec() {
        quote! {
            let __value = reader.read_text(#tag)?;
            #bind.push(#from_str);
        }
    } else {
        quote! {
            let __value = reader.read_text(#tag)?;
            #bind = Some(#from_str);
        }
    };

    quote! {
        #tag => {
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
        Type::CowStr | Type::OptionCowStr | Type::VecCowStr => quote! { __value },
        Type::Bool | Type::OptionBool | Type::VecBool => quote! {
            match &*__value {
                "t" | "true" | "y" | "yes" | "on" | "1" => true,
                "f" | "false" | "n" | "no" | "off" | "0" => false,
                _ => <bool as std::str::FromStr>::from_str(&__value).map_err(|e| XmlError::FromStr(e.into()))?
            }
        },
        Type::T(ty) | Type::OptionT(ty) | Type::VecT(ty) => quote! {
            <#ty as std::str::FromStr>::from_str(&__value).map_err(|e| XmlError::FromStr(e.into()))?
        },
    }
}
