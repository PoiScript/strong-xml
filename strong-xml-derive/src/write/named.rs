use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr};

use crate::types::{Field, Type};

pub fn write(tag: &LitStr, ele_name: &Ident, fields: &Vec<Field>) -> TokenStream {
    let write_attributes = fields.iter().filter_map(|field| match field {
        Field::Attribute { tag, name, ty, .. } => Some(write_attrs(&tag, &name, &ty)),
        _ => None,
    });

    let write_text = fields.iter().filter_map(|field| match field {
        Field::Text { name, ty, .. } => Some(write_text(tag, name, ty)),
        _ => None,
    });

    let write_flatten_text = fields.iter().filter_map(|field| match field {
        Field::FlattenText { tag, name, ty, .. } => Some(write_flatten_text(tag, name, ty)),
        _ => None,
    });

    let write_child = fields.iter().filter_map(|field| match field {
        Field::Child { name, ty, .. } => Some(write_child(name, ty)),
        _ => None,
    });

    let is_leaf_element = fields
        .iter()
        .all(|field| matches!(field, Field::Attribute { .. }));

    let is_text_element = fields
        .iter()
        .any(|field| matches!(field, Field::Text { .. }));

    let can_self_close = fields.iter().all(|field| match field {
        Field::Child { ty, .. } | Field::FlattenText { ty, .. } => ty.is_vec() || ty.is_option(),
        _ => true,
    });

    let content_is_empty = fields.iter().filter_map(|field| match field {
        Field::Child { ty, name, .. } | Field::FlattenText { ty, name, .. } => {
            if ty.is_vec() {
                Some(quote! { #name.is_empty() })
            } else if ty.is_option() {
                Some(quote! { #name.is_none() })
            } else {
                None
            }
        }
        _ => None,
    });

    let write_element_end = if is_leaf_element {
        quote! { write!(&mut writer, "/>")?; }
    } else if is_text_element {
        quote! { #( #write_text )* }
    } else {
        quote! {
            if #can_self_close #( && #content_is_empty )* {
                write!(&mut writer, "/>")?;
            } else {
                write!(&mut writer, ">")?;
                #( #write_child )*
                #( #write_flatten_text )*
                write!(&mut writer, concat!("</", #tag, ">"))?;
            }
        }
    };

    quote! {
        log::debug!(concat!("[", stringify!(#ele_name), "] Started writing."));

        write!(&mut writer, concat!("<", #tag))?;

        #( #write_attributes )*

        #write_element_end

        log::debug!(concat!("[", stringify!(#ele_name), "] Finished writing."));
    }
}

fn write_attrs(tag: &LitStr, name: &Ident, ty: &Type) -> TokenStream {
    let to_str = to_str(ty);

    if ty.is_vec() {
        panic!("`attr` attribute doesn't support Vec.");
    } else if ty.is_option() {
        quote! {
            if let Some(ref __value) = #name {
                write!(&mut writer, concat!(" ", #tag, "=\"{}\""), #to_str)?;
            }
        }
    } else {
        quote! {
            let __value = #name;
            write!(&mut writer, concat!(" ", #tag, "=\"{}\""), #to_str)?;
        }
    }
}

fn write_child(name: &Ident, ty: &Type) -> TokenStream {
    match ty {
        Type::OptionT(_) => quote! {
            if let Some(ref ele) = #name {
                ele.to_writer(&mut writer)?;
            }
        },
        Type::VecT(_) => quote! {
            for ele in #name {
                ele.to_writer(&mut writer)?;
            }
        },
        Type::T(_) => quote! {
            &#name.to_writer(&mut writer)?;
        },
        _ => panic!("`child` attribute only supports Vec<T>, Option<T> and T."),
    }
}

fn write_text(tag: &LitStr, name: &Ident, ty: &Type) -> TokenStream {
    let to_str = to_str(ty);

    quote! {
        write!(&mut writer, ">")?;

        let __value = &#name;

        write!(&mut writer, "{}", strong_xml::utils::xml_escape(#to_str))?;

        write!(&mut writer, concat!("</" , #tag, ">"))?;
    }
}

fn write_flatten_text(tag: &LitStr, name: &Ident, ty: &Type) -> TokenStream {
    let to_str = to_str(ty);

    if ty.is_vec() {
        quote! {
           for __value in #name {
                write!(&mut writer, concat!("<" , #tag, ">"))?;

                write!(&mut writer, "{}", strong_xml::utils::xml_escape(#to_str))?;

                write!(&mut writer, concat!("</" , #tag, ">"))?;
            }
        }
    } else if ty.is_option() {
        quote! {
            if let Some(__value) = #name {
                write!(&mut writer, concat!("<" , #tag, ">"))?;

                write!(&mut writer, "{}", strong_xml::utils::xml_escape(#to_str))?;

                write!(&mut writer, concat!("</" , #tag, ">"))?;
            }
        }
    } else {
        quote! {
            write!(&mut writer, concat!("<" , #tag, ">"))?;

            let __value = &#name;

            write!(&mut writer, "{}", strong_xml::utils::xml_escape(#to_str))?;

            write!(&mut writer, concat!("</" , #tag, ">"))?;
        }
    }
}

fn to_str(ty: &Type) -> TokenStream {
    match &ty {
        Type::CowStr | Type::OptionCowStr | Type::VecCowStr => {
            quote! { __value }
        }
        Type::Bool | Type::OptionBool | Type::VecBool => quote! {
            match __value {
                true => "true",
                false => "false"
            }
        },
        Type::T(_) | Type::OptionT(_) | Type::VecT(_) => {
            quote! { &format!("{}", __value) }
        }
    }
}
