use std::collections::BTreeMap;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{Lit::*, Meta::*, *};

use crate::utils::elide_type_lifetimes;

pub enum Element {
    Struct { name: Ident, fields: Fields },
    Enum { name: Ident, variants: Vec<Fields> },
}

pub enum Fields {
    /// Named fields of a struct or struct variant
    ///
    /// ```ignore
    /// #[xml(tag = "$tag")]
    /// struct $name {
    ///     $( $fields )*
    /// }
    /// ```
    ///
    /// ```ignore
    /// enum Foo {
    ///     #[xml(tag = "$tag")]
    ///     $name {
    ///         $( $fields )*
    ///     }
    /// }
    /// ```
    Named {
        tag: QName,
        name: Ident,
        fields: Vec<Field>,
        namespaces: NamespaceDefs,
    },
    /// Newtype struct or newtype variant
    ///
    /// ```ignore
    /// #[xml($(tag = "$tags",)*)]
    /// struct $name($ty);
    /// ```
    ///
    /// ```ignore
    /// enum Foo {
    ///     #[xml($(tag = "$tags",)*)]
    ///     $name($ty)
    /// }
    /// ```
    Newtype {
        tags: Vec<QName>,
        name: Ident,
        ty: Type,
        namespaces: NamespaceDefs,
    },
}

pub enum Field {
    /// Arrtibute Field
    ///
    /// ```ignore
    /// struct Foo {
    ///     #[xml(attr = "$tag", $default)]
    ///     $name: $ty,
    /// }
    /// ```
    Attribute {
        name: TokenStream,
        bind: Ident,
        ty: Type,
        tag: QName,
        default: bool,
    },
    /// Child(ren) Field
    ///
    /// ```ignore
    /// struct Foo {
    ///     #[xml(child = "$tag", child = "$tag", $default)]
    ///     $name: $ty,
    /// }
    /// ```
    Child {
        name: TokenStream,
        bind: Ident,
        ty: Type,
        default: bool,
        tags: Vec<QName>,
        namespaces: NamespaceDefs,
    },
    /// Text Field
    ///
    /// ```ignore
    /// struct Foo {
    ///     #[xml(text, $default)]
    ///     $name: $ty,
    /// }
    /// ```
    Text {
        name: TokenStream,
        bind: Ident,
        ty: Type,
        is_cdata: bool,
    },
    /// Flatten Text
    ///
    /// ```ignore
    /// struct Foo {
    ///     #[xml(flatten_text = "$tag", $default)]
    ///     $name: $ty,
    /// }
    /// ```
    FlattenText {
        name: TokenStream,
        bind: Ident,
        ty: Type,
        default: bool,
        tag: QName,
        is_cdata: bool,
    },
}

pub enum Type {
    // Cow<'a, str>
    CowStr,
    // Option<Cow<'a, str>>
    OptionCowStr,
    // Vec<Cow<'a, str>>
    VecCowStr,
    // T
    T(syn::Type),
    // Option<T>
    OptionT(syn::Type),
    // Vec<T>
    VecT(syn::Type),
    // bool
    Bool,
    // Vec<bool>
    VecBool,
    // Option<bool>
    OptionBool,
}

#[derive(Clone)]
pub enum QName {
    Prefixed(LitStr),
    Unprefixed(LitStr),
}

pub struct NamespaceDef {
    prefix: Option<String>,
    namespace: String,
}

pub type NamespaceDefs = BTreeMap<Option<String>, NamespaceDef>;

impl Element {
    pub fn parse(input: DeriveInput) -> Element {
        match input.data {
            Data::Struct(data) => Element::Struct {
                name: input.ident.clone(),
                fields: Fields::parse(data.fields, input.attrs, input.ident),
            },
            Data::Enum(data) => Element::Enum {
                name: input.ident,
                variants: data
                    .variants
                    .into_iter()
                    .map(|variant| Fields::parse(variant.fields, variant.attrs, variant.ident))
                    .collect::<Vec<_>>(),
            },
            Data::Union(_) => panic!("strong-xml doesn't support Union."),
        }
    }
}

impl Fields {
    pub fn parse(fields: syn::Fields, attrs: Vec<Attribute>, name: Ident) -> Fields {
        // Finding `tag` attribute
        let mut tags = Vec::new();
        let mut namespaces: NamespaceDefs = BTreeMap::default();

        for meta in attrs.into_iter().filter_map(get_xml_meta).flatten() {
            match meta {
                NestedMeta::Meta(NameValue(m)) if m.path.is_ident("tag") => {
                    if let Str(lit) = m.lit {
                        match QName::parse(lit) {
                            Ok(q) => tags.push(q),
                            Err(e) => panic!("{}", e),
                        }
                    } else {
                        panic!("Expected a string literal.");
                    }
                }
                NestedMeta::Meta(NameValue(MetaNameValue { lit, path, .. }))
                    if path.is_ident("ns") =>
                {
                    let (prefix, namespace) = if let Str(lit) = lit {
                        if let Some((pfx, ns)) = lit.value().split_once(": ") {
                            (Some(pfx.to_string()), ns.to_string())
                        } else {
                            (None, lit.value().to_string())
                        }
                    } else {
                        panic!("Expected a string literal.");
                    };

                    if namespaces.contains_key(&prefix) {
                        if let Some(ref prefix) = prefix {
                            panic!("namespace {} already defined", prefix);
                        } else {
                            panic!("default namespace already defined");
                        };
                    }

                    if let Some(prefix) = &prefix {
                        if prefix.contains(":") {
                            panic!("prefix cannot contain `:`");
                        }

                        if prefix == "xml" && namespace != "http://www.w3.org/XML/1998/namespace" {
                            panic!("xml prefix can only be bound to http://www.w3.org/XML/1998/namespace");
                        } else if prefix.starts_with("xml") {
                            panic!("prefix cannot start with `xml`");
                        }
                    }
                    namespaces.insert(prefix.clone(), NamespaceDef { prefix, namespace });
                }
                _ => (),
            }
        }

        if tags.is_empty() {
            panic!("Missing `tag` attribute.");
        }

        match fields {
            syn::Fields::Unit => Fields::Named {
                name,
                tag: tags.remove(0),
                namespaces,
                fields: Vec::new(),
            },
            syn::Fields::Unnamed(fields) => {
                // we will assume it's a newtype stuct/enum
                // if it has only one field and no field attribute
                if fields.unnamed.len() == 1 {
                    let field = fields.unnamed.first().unwrap().clone();
                    if field.attrs.into_iter().filter_map(get_xml_meta).count() == 0 {
                        return Fields::Newtype {
                            name,
                            tags,
                            ty: Type::parse(field.ty),
                            namespaces,
                        };
                    }
                }

                Fields::Named {
                    name,
                    tag: tags.remove(0),
                    namespaces,
                    fields: fields
                        .unnamed
                        .into_iter()
                        .enumerate()
                        .map(|(index, field)| {
                            let index = syn::Index::from(index);
                            let bind = format_ident!("__self_{}", index);
                            Field::parse(quote!(#index), bind, field)
                        })
                        .collect::<Vec<_>>(),
                }
            }
            syn::Fields::Named(_) => Fields::Named {
                name,
                tag: tags.remove(0),
                namespaces,
                fields: fields
                    .into_iter()
                    .map(|field| {
                        let name = field.ident.clone().unwrap();
                        let bind = format_ident!("__self_{}", name);
                        Field::parse(quote!(#name), bind, field)
                    })
                    .collect::<Vec<_>>(),
            },
        }
    }
}

impl Field {
    pub fn parse(name: TokenStream, bind: Ident, field: syn::Field) -> Field {
        let mut default = false;
        let mut attr_tag = None;
        let mut child_tags = Vec::new();
        let mut is_text = false;
        let mut flatten_text_tag = None;
        let mut is_cdata = false;

        for meta in field.attrs.into_iter().filter_map(get_xml_meta).flatten() {
            match meta {
                NestedMeta::Meta(Path(p)) if p.is_ident("default") => {
                    if default {
                        panic!("Duplicate `default` attribute.");
                    } else {
                        default = true;
                    }
                }
                NestedMeta::Meta(NameValue(m)) if m.path.is_ident("attr") => {
                    if let Str(lit) = m.lit {
                        if attr_tag.is_some() {
                            panic!("Duplicate `attr` attribute.");
                        } else if is_text {
                            panic!("`attr` attribute and `text` attribute is disjoint.");
                        } else if is_cdata {
                            panic!("`attr` attribute and `cdata` attribute is disjoint.")
                        } else if !child_tags.is_empty() {
                            panic!("`attr` attribute and `child` attribute is disjoint.");
                        } else if flatten_text_tag.is_some() {
                            panic!("`attr` attribute and `flatten_text` attribute is disjoint.");
                        } else {
                            match QName::parse(lit) {
                                Ok(q) => attr_tag = Some(q),
                                Err(e) => panic!("{}", e),
                            }
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
                NestedMeta::Meta(Path(ref p)) if p.is_ident("cdata") => {
                    if is_cdata {
                        panic!("Duplicate `cdata` attribute.");
                    } else if attr_tag.is_some() {
                        panic!("`text` attribute and `attr` attribute is disjoint.");
                    } else if !child_tags.is_empty() {
                        panic!("`text` attribute and `child` attribute is disjoint.");
                    } else {
                        is_cdata = true;
                    }
                }
                NestedMeta::Meta(NameValue(m)) if m.path.is_ident("child") => {
                    if let Str(lit) = m.lit {
                        if is_text {
                            panic!("`child` attribute and `text` attribute is disjoint.");
                        } else if attr_tag.is_some() {
                            panic!("`child` attribute and `attr` attribute is disjoint.");
                        } else if is_cdata {
                            panic!("`child` attribute and `cdata` attribute is disjoint.")
                        } else if flatten_text_tag.is_some() {
                            panic!("`child` attribute and `flatten_text` attribute is disjoint.");
                        } else {
                            match QName::parse(lit) {
                                Ok(q) => child_tags.push(q),
                                Err(e) => panic!("{}", e),
                            }
                        }
                    } else {
                        panic!("Expected a string literal.");
                    }
                }
                NestedMeta::Meta(NameValue(m)) if m.path.is_ident("flatten_text") => {
                    if let Str(lit) = m.lit {
                        if is_text {
                            panic!("`flatten_text` attribute and `text` attribute is disjoint.");
                        } else if !child_tags.is_empty() {
                            panic!("`flatten_text` attribute and `child` attribute is disjoint.");
                        } else if attr_tag.is_some() {
                            panic!("`flatten_text` attribute and `attr` attribute is disjoint.");
                        } else if flatten_text_tag.is_some() {
                            panic!("Duplicate `flatten_text` attribute.");
                        } else {
                            match QName::parse(lit) {
                                Ok(q) => flatten_text_tag = Some(q),
                                Err(e) => panic!("{}", e),
                            }
                        }
                    } else {
                        panic!("Expected a string literal.");
                    }
                }
                NestedMeta::Meta(NameValue(m))
                    if m.path
                        .get_ident()
                        .filter(|ident| ident.to_string().starts_with("ns"))
                        .is_some() =>
                {
                    panic!("Namespace declaration not supported in this position");
                }
                _ => (),
            }
        }

        if let Some(tag) = attr_tag {
            Field::Attribute {
                name,
                bind,
                ty: Type::parse(field.ty),
                tag,
                default,
            }
        } else if !child_tags.is_empty() {
            Field::Child {
                name,
                bind,
                ty: Type::parse(field.ty),
                default,
                tags: child_tags,
                namespaces: NamespaceDefs::new(),
            }
        } else if is_text {
            Field::Text {
                name,
                bind,
                ty: Type::parse(field.ty),
                is_cdata,
            }
        } else if let Some(tag) = flatten_text_tag {
            Field::FlattenText {
                name,
                bind,
                ty: Type::parse(field.ty),
                default,
                tag,
                is_cdata,
            }
        } else {
            panic!("Field should have one of `attr`, `child`, `text` or `flatten_text` attribute.");
        }
    }
}

impl Type {
    pub fn is_option(&self) -> bool {
        matches!(
            self,
            Type::OptionCowStr | Type::OptionT(_) | Type::OptionBool
        )
    }

    pub fn is_vec(&self) -> bool {
        matches!(self, Type::VecCowStr | Type::VecT(_) | Type::VecBool)
    }

    fn parse(mut ty: syn::Type) -> Self {
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
                match &args[0] {
                    GenericArgument::Type(arg) => Some(arg),
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

        elide_type_lifetimes(&mut ty);

        if let Some(ty) = is_vec(&ty) {
            if is_cow_str(&ty) {
                Type::VecCowStr
            } else if is_bool(&ty) {
                Type::VecBool
            } else {
                Type::VecT(ty.clone())
            }
        } else if let Some(ty) = is_option(&ty) {
            if is_cow_str(&ty) {
                Type::OptionCowStr
            } else if is_bool(&ty) {
                Type::OptionBool
            } else {
                Type::OptionT(ty.clone())
            }
        } else if is_cow_str(&ty) {
            Type::CowStr
        } else if is_bool(&ty) {
            Type::Bool
        } else {
            Type::T(ty)
        }
    }
}
use std::io::{Error, ErrorKind, Result};

impl QName {
    pub fn parse(name: LitStr) -> Result<QName> {
        let space_count = name.value().matches(' ').count();
        if space_count > 0 {
            return Err(Error::new(ErrorKind::Other, "QName can not contain spaces"));
        }

        let colon_count = name.value().matches(':').count();
        match colon_count {
            0 => Ok(QName::Unprefixed(name)),
            1 => Ok(QName::Prefixed(name)),
            _ => Err(Error::new(
                ErrorKind::Other,
                "QName can only have a max of 1 colon",
            )),
        }
    }

    pub fn prefix(&self) -> Option<String> {
        match self {
            Self::Prefixed(name) => name
                .value()
                .split_once(":")
                .map(|(prefix, _)| prefix.to_owned()),
            Self::Unprefixed(_) => None,
        }
    }

    pub fn local(&self) -> String {
        match self {
            Self::Prefixed(name) => name.value().split_once(":").unwrap().1.to_owned(),
            Self::Unprefixed(name) => name.value(),
        }
    }
}

impl ToString for QName {
    fn to_string(&self) -> String {
        match self {
            QName::Prefixed(tag) | QName::Unprefixed(tag) => tag.value(),
        }
    }
}

impl ToTokens for QName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            QName::Prefixed(tag) | QName::Unprefixed(tag) => tag.to_tokens(tokens),
        }
    }
}

impl NamespaceDef {
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }
}

fn get_xml_meta(attr: Attribute) -> Option<Vec<NestedMeta>> {
    if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "xml" {
        match attr.parse_meta() {
            Ok(Meta::List(meta)) => Some(meta.nested.iter().cloned().collect()),
            _ => None,
        }
    } else {
        None
    }
}
