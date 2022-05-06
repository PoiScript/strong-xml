use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr};

use crate::types::{Field, Type};

pub fn write(tag: &LitStr, prefix: &Option<LitStr>, ele_name: TokenStream, fields: &[Field]) -> TokenStream {

    let write_attributes = fields.iter().filter_map(|field| match field {
        Field::Attribute { tag, prefix, bind, ty, .. } => Some(write_attrs(&tag, prefix, &bind, &ty, &ele_name)),
        _ => None,
    });

    let write_text = fields.iter().filter_map(|field| match field {
        Field::Text {
            bind, ty, is_cdata,..
        } => Some(write_text(tag, prefix, bind, ty, &ele_name, *is_cdata)),
        _ => None,
    });

    let write_flatten_text = fields.iter().filter_map(|field| match field {
        Field::FlattenText {
            tag,
            bind,
            ty,
            is_cdata,
            ..
        } => Some(write_flatten_text(tag, prefix, bind, ty, &ele_name, *is_cdata)),
        _ => None,
    });

    let write_child = fields.iter().filter_map(|field| match field {
        Field::Child { bind, ty, .. } => Some(write_child(bind, ty, &ele_name)),
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
        Field::Child { ty, bind, .. } | Field::FlattenText { ty, bind, .. } => {
            if ty.is_vec() {
                Some(quote! { #bind.is_empty() })
            } else if ty.is_option() {
                Some(quote! { #bind.is_none() })
            } else {
                None
            }
        }
        _ => None,
    });

    let write_element_end = if is_leaf_element {
        quote! { writer.write_element_end_empty()?; }
    } else if is_text_element {
        quote! { #( #write_text )* }
    } else {
        quote! {
            if #can_self_close #( && #content_is_empty )* {
                writer.write_element_end_empty()?;
            } else {
                writer.write_element_end_open()?;
                #( #write_child )*
                #( #write_flatten_text )*
                writer.write_element_end_close(#prefix, #tag)?;
            }
        }
    };

    quote! {
        strong_xml::log_start_writing!(#ele_name);

        writer.write_element_start(#prefix, #tag)?;

        #( #write_attributes )*

        #write_element_end

        strong_xml::log_finish_writing!(#ele_name);
    }
}

fn write_attrs(tag: &LitStr, prefix: &Option<LitStr>, name: &Ident, ty: &Type, ele_name: &TokenStream) -> TokenStream {
    let to_str = to_str(ty);

    let prefix = if let Some(prefix) = prefix {
        quote!(Some(#prefix))
    } else {
        quote!(None)
    };

    if ty.is_vec() {
        panic!("`attr` attribute doesn't support Vec.");
    } else if ty.is_option() {
        quote! {
            strong_xml::log_start_writing_field!(#ele_name, #name);

            if let Some(__value) = #name {
                writer.write_attribute(#prefix, #tag, #to_str)?;
            }

            strong_xml::log_finish_writing_field!(#ele_name, #name);
        }
    } else {
        quote! {
            strong_xml::log_start_writing_field!(#ele_name, #name);

            let __value = #name;
            writer.write_attribute(#prefix, #tag, #to_str)?;

            strong_xml::log_finish_writing_field!(#ele_name, #name);
        }
    }
}

fn write_child(name: &Ident, ty: &Type, ele_name: &TokenStream) -> TokenStream {
    match ty {
        Type::OptionT(_) => quote! {
            strong_xml::log_start_writing_field!(#ele_name, #name);

            if let Some(ref ele) = #name {
                ele.to_writer(&mut writer)?;
            }

            strong_xml::log_finish_writing_field!(#ele_name, #name);
        },
        Type::VecT(_) => quote! {
            strong_xml::log_start_writing_field!(#ele_name, #name);

            for ele in #name {
                ele.to_writer(&mut writer)?;
            }

            strong_xml::log_finish_writing_field!(#ele_name, #name);
        },
        Type::T(_) => quote! {
            strong_xml::log_start_writing_field!(#ele_name, #name);

            #name.to_writer(&mut writer)?;

            strong_xml::log_finish_writing_field!(#ele_name, #name);
        },
        _ => panic!("`child` attribute only supports Vec<T>, Option<T> and T."),
    }
}

fn write_text(
    tag: &LitStr,
    prefix: &Option<LitStr>,
    name: &Ident,
    ty: &Type,
    ele_name: &TokenStream,
    is_cdata: bool,
) -> TokenStream {
    let to_str = to_str(ty);
    let wrtie_fn = if is_cdata {
        quote!(write_cdata_text)
    } else {
        quote!(write_text)
    };

    let prefix = if let Some(prefix) = prefix {
        quote!(Some(#prefix))
    } else {
        quote!(None)
    };

    quote! {
        writer.write_element_end_open()?;

        strong_xml::log_start_writing_field!(#ele_name, #name);

        let __value = &#name;

        writer.#wrtie_fn(#to_str)?;

        strong_xml::log_finish_writing_field!(#ele_name, #name);

        writer.write_element_end_close(#prefix, #tag)?;
    }
}

fn write_flatten_text(
    tag: &LitStr,
    prefix: &Option<LitStr>,
    name: &Ident,
    ty: &Type,
    ele_name: &TokenStream,
    is_cdata: bool,
) -> TokenStream {
    let to_str = to_str(ty);

    let prefix = if let Some(prefix) = prefix {
        quote!(Some(#prefix))
    } else {
        quote!(None)
    };

    if ty.is_vec() {
        quote! {
            strong_xml::log_finish_writing_field!(#ele_name, #name);

            for __value in #name {
                writer.write_flatten_text(#prefix, #tag, #to_str, #is_cdata)?;
            }

            strong_xml::log_finish_writing_field!(#ele_name, #name);
        }
    } else if ty.is_option() {
        quote! {
            strong_xml::log_finish_writing_field!(#ele_name, #name);

            if let Some(__value) = #name {
                writer.write_flatten_text(#prefix, #tag, #to_str, #is_cdata)?;
            }

            strong_xml::log_finish_writing_field!(#ele_name, #name);
        }
    } else {
        quote! {
            strong_xml::log_finish_writing_field!(#ele_name, #name);

            let __value = &#name;
            writer.write_flatten_text(#prefix, #tag, #to_str, #is_cdata)?;

            strong_xml::log_finish_writing_field!(#ele_name, #name);
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
