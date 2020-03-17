use proc_macro2::Span;

use syn::{Lit::*, Meta::*, *};

#[allow(clippy::large_enum_variant)]
pub enum Element {
    Enum(EnumElement),
    Struct(StructElement),
}

pub struct EnumElement {
    pub name: Ident,
    pub elements: Vec<Variant>,
}

pub struct Variant {
    pub name: Ident,
    pub ty: syn::Type,
    pub tags: Vec<LitStr>,
}

pub struct StructElement {
    pub name: Ident,
    pub tag: LitStr,
    pub extend_attrs: Option<Ident>,
    pub attributes: Vec<AttributeField>,
    pub children: Vec<ChildrenField>,
    pub flatten_text: Vec<FlattenTextField>,
    pub text: Option<TextField>,
}

// #[xml(attr = "attr", default)]
// key: Vec<_>
pub struct AttributeField {
    pub name: Ident,
    pub ty: Type,
    pub tag: LitStr,
    pub default: bool,
}

// #[xml(child = "t1", child = "t2", default)]
// key: Vec<_>
pub struct ChildrenField {
    pub name: Ident,
    pub ty: Type,
    pub default: bool,
    pub tags: Vec<LitStr>,
}

// #[xml(faltten_text = "t", default)]
// key: Vec<_>
pub struct FlattenTextField {
    pub name: Ident,
    pub ty: Type,
    pub default: bool,
    pub tag: LitStr,
}

// #[xml(text)]
// key: Cow<>
pub struct TextField {
    pub name: Ident,
    pub ty: Type,
}

impl Element {
    pub fn parse(input: &DeriveInput) -> Element {
        match input.data {
            Data::Struct(ref data) => Self::parse_struct(data, &input.attrs, &input.ident),
            Data::Enum(ref data) => Self::parse_enum(data, &input.ident),
            Data::Union(_) => panic!("#[derive(Xml)] doesn't support Union."),
        }
    }

    pub fn parse_struct(data: &DataStruct, attrs: &[Attribute], ident: &Ident) -> Element {
        let mut tag = None;
        let mut extend_attrs = None;

        for meta in attrs.iter().filter_map(get_xml_meta).flatten() {
            match meta {
                NestedMeta::Meta(NameValue(ref m)) if m.path.is_ident("tag") => {
                    if let Str(ref lit) = m.lit {
                        if tag.is_some() {
                            panic!("Duplicate `tag` attribute.");
                        } else {
                            tag = Some(lit.clone());
                        }
                    } else {
                        panic!("Expected a string literal.");
                    }
                }
                NestedMeta::Meta(NameValue(ref m)) if m.path.is_ident("extend_attrs") => {
                    if let Str(ref lit) = m.lit {
                        if extend_attrs.is_some() {
                            panic!("Duplicate `extend_attrs` attribute.");
                        } else {
                            extend_attrs = Some(Ident::new(&lit.value(), Span::call_site()));
                        }
                    } else {
                        panic!("Expected a string literal.");
                    }
                }
                _ => (),
            }
        }

        let tag = tag.expect("Missing `tag` attribute.");

        let mut attributes = Vec::new();
        let mut text = None;
        let mut children = Vec::new();
        let mut flatten_text = Vec::new();

        for field in data.fields.iter() {
            let mut default = false;
            let mut attr_tag = None;
            let mut child_tags = Vec::new();
            let mut is_text = false;
            let mut flatten_text_tag = None;

            for meta in field.attrs.iter().filter_map(get_xml_meta).flatten() {
                match meta {
                    NestedMeta::Meta(Path(ref p)) if p.is_ident("default") => {
                        if default {
                            panic!("Duplicate `default` attribute.");
                        } else {
                            default = true;
                        }
                    }
                    NestedMeta::Meta(NameValue(ref m)) if m.path.is_ident("attr") => {
                        if let Str(ref lit) = m.lit {
                            if attr_tag.is_some() {
                                panic!("Duplicate `attr` attribute.");
                            } else if is_text {
                                panic!("`attr` attribute and `text` attribute is disjoint.");
                            } else if !child_tags.is_empty() {
                                panic!("`attr` attribute and `child` attribute is disjoint.");
                            } else if flatten_text_tag.is_some() {
                                panic!(
                                    "`attr` attribute and `flatten_text` attribute is disjoint."
                                );
                            } else {
                                attr_tag = Some(lit.clone());
                            }
                        } else {
                            panic!("Expected a string literal.");
                        }
                    }
                    NestedMeta::Meta(Path(ref p)) if p.is_ident("text") => {
                        if is_text {
                            panic!("Duplicate `text` attribute.");
                        } else if attr_tag.is_some() {
                            panic!("`text` attribute and `attr` attribute is disjoint.");
                        } else if !child_tags.is_empty() {
                            panic!("`text` attribute and `child` attribute is disjoint.");
                        } else if flatten_text_tag.is_some() {
                            panic!("`text` attribute and `flatten_text` attribute is disjoint.");
                        } else {
                            is_text = true;
                        }
                    }
                    NestedMeta::Meta(NameValue(ref m)) if m.path.is_ident("child") => {
                        if let Str(ref lit) = m.lit {
                            if is_text {
                                panic!("`child` attribute and `text` attribute is disjoint.");
                            } else if attr_tag.is_some() {
                                panic!("`child` attribute and `attr` attribute is disjoint.");
                            } else if flatten_text_tag.is_some() {
                                panic!(
                                    "`child` attribute and `flatten_text` attribute is disjoint."
                                );
                            } else {
                                child_tags.push(lit.clone());
                            }
                        } else {
                            panic!("Expected a string literal.");
                        }
                    }
                    NestedMeta::Meta(NameValue(ref m)) if m.path.is_ident("flatten_text") => {
                        if let Str(ref lit) = m.lit {
                            if is_text {
                                panic!(
                                    "`flatten_text` attribute and `text` attribute is disjoint."
                                );
                            } else if !child_tags.is_empty() {
                                panic!(
                                    "`flatten_text` attribute and `child` attribute is disjoint."
                                );
                            } else if attr_tag.is_some() {
                                panic!(
                                    "`flatten_text` attribute and `attr` attribute is disjoint."
                                );
                            } else if flatten_text_tag.is_some() {
                                panic!("Duplicate `flatten_text` attribute.");
                            } else {
                                flatten_text_tag = Some(lit.clone());
                            }
                        } else {
                            panic!("Expected a string literal.");
                        }
                    }
                    _ => (),
                }
            }

            if let Some(tag) = attr_tag {
                attributes.push(AttributeField {
                    name: field.ident.clone().unwrap(),
                    ty: (&field.ty).into(),
                    tag,
                    default,
                });
            } else if !child_tags.is_empty() {
                children.push(ChildrenField {
                    name: field.ident.clone().unwrap(),
                    ty: (&field.ty).into(),
                    default,
                    tags: child_tags,
                });
            } else if is_text {
                if text.is_some() {
                    panic!("Duplicate `text` field.");
                }
                text = Some(TextField {
                    name: field.ident.clone().unwrap(),
                    ty: (&field.ty).into(),
                });
            } else if let Some(tag) = flatten_text_tag {
                flatten_text.push(FlattenTextField {
                    name: field.ident.clone().unwrap(),
                    ty: (&field.ty).into(),
                    default,
                    tag,
                });
            } else {
                panic!(
                    "Field should have one of `attr`, `child`, `text` or `flatten_text` attribute."
                );
            }
        }

        Element::Struct(StructElement {
            name: ident.clone(),
            tag,
            extend_attrs,
            attributes,
            children,
            flatten_text,
            text,
        })
    }

    pub fn parse_enum(data: &DataEnum, ident: &Ident) -> Element {
        let mut elements = Vec::new();

        for variant in &data.variants {
            let name = &variant.ident;
            let ty = &variant.fields.iter().next().unwrap().ty;

            let mut tags = Vec::new();

            for meta in variant.attrs.iter().filter_map(get_xml_meta).flatten() {
                match meta {
                    NestedMeta::Meta(NameValue(ref m)) if m.path.is_ident("tag") => {
                        if let Str(ref lit) = m.lit {
                            tags.push(lit.clone());
                        } else {
                            panic!("Expected a string literal.");
                        }
                    }
                    _ => (),
                }
            }

            if tags.is_empty() {
                panic!("Missing `tag` attribute.")
            }

            elements.push(Variant {
                name: name.clone(),
                ty: ty.clone(),
                tags,
            });
        }

        Element::Enum(EnumElement {
            name: ident.clone(),
            elements,
        })
    }
}

fn get_xml_meta(attr: &Attribute) -> Option<Vec<NestedMeta>> {
    if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "xml" {
        match attr.parse_meta() {
            Ok(Meta::List(meta)) => Some(meta.nested.iter().cloned().collect()),
            _ => None,
        }
    } else {
        None
    }
}

pub enum Type {
    // Vec<Cow<'a, str>>, flatten_text
    VecCowStr,
    // Vec<T>, children
    VecT(syn::Type),
    // Option<T>, children, attr
    OptionT(syn::Type),
    // Option<Cow<'a, str>>, flatten_text, attr
    OptionCowStr,
    // Option<bool>, attr
    OptionBool,
    // Option<usize>, attr
    OptionUsize,
    // Cow<'a, str>, flatten_text
    CowStr,
    // bool, attr
    Bool,
    // usize, attr
    Usize,
    // T, child, attr
    T(syn::Type),
}

impl From<&syn::Type> for Type {
    fn from(ty: &syn::Type) -> Self {
        if let Some(ty) = is_vec(ty) {
            if is_cow_str(ty) {
                Type::VecCowStr
            } else {
                Type::VecT(ty.clone())
            }
        } else if let Some(ty) = is_option(ty) {
            if is_cow_str(ty) {
                Type::OptionCowStr
            } else if is_bool(ty) {
                Type::OptionBool
            } else if is_usize(ty) {
                Type::OptionUsize
            } else {
                Type::OptionT(ty.clone())
            }
        } else if is_cow_str(ty) {
            Type::CowStr
        } else if is_usize(ty) {
            Type::Usize
        } else if is_bool(ty) {
            Type::Bool
        } else {
            Type::T(ty.clone())
        }
    }
}

fn is_vec(ty: &syn::Type) -> Option<&syn::Type> {
    let path = match ty {
        syn::Type::Path(ty) => &ty.path,
        _ => return None,
    };
    let seg = path.segments.last()?;
    let args = match &seg.arguments {
        PathArguments::AngleBracketed(bracketed) => &bracketed.args,
        _ => return None,
    };
    if seg.ident == "Vec" && args.len() == 1 {
        match args[0] {
            GenericArgument::Type(ref arg) => Some(arg),
            _ => None,
        }
    } else {
        None
    }
}

fn is_option(ty: &syn::Type) -> Option<&syn::Type> {
    let path = match ty {
        syn::Type::Path(ty) => &ty.path,
        _ => return None,
    };
    let seg = path.segments.last()?;
    let args = match &seg.arguments {
        PathArguments::AngleBracketed(bracketed) => &bracketed.args,
        _ => return None,
    };
    if seg.ident == "Option" && args.len() == 1 {
        match args[0] {
            GenericArgument::Type(ref arg) => Some(arg),
            _ => None,
        }
    } else {
        None
    }
}

fn is_cow_str(ty: &syn::Type) -> bool {
    let path = match ty {
        syn::Type::Path(ty) => &ty.path,
        _ => return false,
    };
    let seg = match path.segments.last() {
        Some(seg) => seg,
        None => return false,
    };
    let args = match &seg.arguments {
        PathArguments::AngleBracketed(bracketed) => &bracketed.args,
        _ => return false,
    };
    if seg.ident == "Cow" && args.len() == 2 {
        match &args[1] {
            GenericArgument::Type(syn::Type::Path(ty)) => ty.path.is_ident("str"),
            _ => false,
        }
    } else {
        false
    }
}

fn is_bool(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(ty) if ty.path.is_ident("bool"))
}

fn is_usize(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(ty) if ty.path.is_ident("usize"))
}

pub fn trim_lifetime(ty: &syn::Type) -> Option<&Ident> {
    let path = match ty {
        syn::Type::Path(ty) => &ty.path,
        _ => return None,
    };
    let seg = path.segments.last()?;
    Some(&seg.ident)
}
