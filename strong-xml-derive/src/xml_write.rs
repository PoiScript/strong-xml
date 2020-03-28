use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr};

use crate::types::{Element, EnumElement, StructElement, Type};

pub fn write(element: &Element) -> TokenStream {
    match element {
        Element::Enum(enum_ele) => write_enum_ele(enum_ele),
        Element::Struct(struct_ele) => write_struct_ele(struct_ele),
    }
}

fn write_struct_ele(struct_ele: &StructElement) -> TokenStream {
    let ele_name = &struct_ele.name;
    let tag = &struct_ele.tag;
    let attr_fields = &struct_ele.attributes;
    let child_fields = &struct_ele.children;
    let flatten_text_fields = &struct_ele.flatten_text;
    let text_field = &struct_ele.text;

    let extend_attrs = &struct_ele
        .extend_attrs
        .as_ref()
        .map(|extend_attrs| quote! { #extend_attrs(&self, &mut writer)?; });

    let write_attr_fields = attr_fields
        .iter()
        .map(|e| write_attrs(&e.tag, &e.name, &e.ty));

    let is_leaf_element =
        child_fields.is_empty() && flatten_text_fields.is_empty() && text_field.is_none();

    let can_be_self_close = child_fields
        .iter()
        .map(|e| &e.ty)
        .chain(flatten_text_fields.iter().map(|e| &e.ty))
        .all(|ty| matches!(ty, Type::VecCowStr | Type::VecT(_) | Type::OptionT(_) | Type::OptionCowStr | Type::OptionBool | Type::OptionUsize));

    let write_element_end = if is_leaf_element {
        quote! { write!(&mut writer, "/>")?; }
    } else if let Some(text_field) = text_field {
        let name = &text_field.name;
        quote! {
            write!(&mut writer, concat!(">{}</", #tag, ">"), strong_xml::utils::xml_escape(&self.#name.to_string()))?;
        }
    } else {
        let content_is_empty = child_fields
            .iter()
            .map(|e| (&e.name, &e.ty))
            .chain(flatten_text_fields.iter().map(|e| (&e.name, &e.ty)))
            .flat_map(|(name, ty)| match ty {
                Type::VecCowStr | Type::VecT(_) => Some(quote! { self.#name.is_empty() }),
                Type::OptionT(_) | Type::OptionCowStr | Type::OptionBool | Type::OptionUsize => {
                    Some(quote! { self.#name.is_none() })
                }
                _ => None,
            });

        let write_child_fields = child_fields.iter().map(|e| write_child(&e.name, &e.ty));

        let write_flatten_text_fields = flatten_text_fields
            .iter()
            .map(|e| write_flatten_text(&e.tag, &e.name, &e.ty));

        quote! {
            if #can_be_self_close #( && #content_is_empty )* {
                write!(&mut writer, "/>")?;
            } else {
                write!(&mut writer, ">")?;
                #( #write_child_fields )*
                #( #write_flatten_text_fields )*
                write!(&mut writer, concat!("</", #tag, ">"))?;
            }
        }
    };

    quote! {
        log::debug!(concat!("[", stringify!(#ele_name), "] Started writing."));

        write!(&mut writer, concat!("<", #tag))?;

        #( #write_attr_fields )*

        #extend_attrs

        #write_element_end

        log::debug!(concat!("[", stringify!(#ele_name), "] Finished writing."));
    }
}

fn write_attrs(tag: &LitStr, name: &Ident, ty: &Type) -> TokenStream {
    match ty {
        Type::OptionCowStr | Type::OptionBool | Type::OptionUsize => quote! {
            if let Some(ref value) = self.#name {
                write!(&mut writer, concat!(" ", #tag, "=\"{}\""), value)?;
            }
        },
        Type::CowStr | Type::Bool | Type::Usize | Type::T(_) => quote! {
            write!(&mut writer, concat!(" ", #tag, "=\"{}\""), self.#name)?;
        },
        Type::OptionT(_) => quote! {
            if let Some(ref value) = self.#name {
                write!(&mut writer, concat!(" ", #tag, "=\"{}\""), value)?;
            }
        },
        _ => panic!(
            "#[xml(attr = \"\")] only supports Cow<str>, Option<Cow<str>>, bool, Option<bool>, usize, Option<usize> and Option<T>."
        ),
    }
}

fn write_child(name: &Ident, ty: &Type) -> TokenStream {
    match ty {
        Type::OptionT(_) => quote! {
            if let Some(ref ele) = self.#name {
                ele.to_writer(&mut writer)?;
            }
        },
        Type::VecT(_) => quote! {
            for ele in &self.#name {
                ele.to_writer(&mut writer)?;
            }
        },
        Type::T(_) => quote! {
            &self.#name.to_writer(&mut writer)?;
        },
        _ => panic!("#[xml(child = \"\")] only support Vec<T>, Option<T> and T."),
    }
}

fn write_flatten_text(tag: &LitStr, name: &Ident, ty: &Type) -> TokenStream {
    match ty {
        Type::CowStr => quote! {
            write!(&mut writer, concat!("<" , #tag, ">"))?;
            write!(&mut writer, "{}", strong_xml::utils::xml_escape(&self.#name))?;
            write!(&mut writer, concat!("</" , #tag, ">"))?;
        },
        Type::OptionCowStr => quote! {
            if let Some(value) = &self.#name {
                write!(&mut writer, concat!("<" , #tag, ">"))?;
                write!(&mut writer, "{}", strong_xml::utils::xml_escape(&value))?;
                write!(&mut writer, concat!("</" , #tag, ">"))?;
            }
        },
        Type::VecCowStr => quote! {
           for value in &self.#name {
                write!(&mut writer, concat!("<" , #tag, ">"))?;
                write!(&mut writer, "{}", strong_xml::utils::xml_escape(&value))?;
                write!(&mut writer, concat!("</" , #tag, ">"))?;
            }
        },
        Type::Bool | Type::Usize => quote! {
            write!(&mut writer, concat!("<" , #tag, ">"))?;
            write!(&mut writer, "{}", strong_xml::utils::xml_escape(&self.#name.to_string()))?;
            write!(&mut writer, concat!("</" , #tag, ">"))?;
        },
        Type::OptionBool | Type::OptionUsize => quote! {
            if let Some(ref value) = self.#name {
                write!(&mut writer, concat!("<" , #tag, ">"))?;
                write!(&mut writer, "{}", strong_xml::utils::xml_escape(&value.to_string()))?;
                write!(&mut writer, concat!("</" , #tag, ">"))?;
            }
        },
        _ => panic!(
            "#[xml(flatten_text)] only support bool, Option<bool>, usize, Option<usize>, Cow<str>, Vec<Cow<str>> and Option<Cow<str>>."
        ),
    }
}

fn write_enum_ele(enum_ele: &EnumElement) -> TokenStream {
    let name = &enum_ele.name;
    let var_names = enum_ele.elements.iter().map(|var| &var.name);

    quote! {
        match self {
            #( #name::#var_names(s) => s.to_writer(writer)?, )*
        }
    }
}
