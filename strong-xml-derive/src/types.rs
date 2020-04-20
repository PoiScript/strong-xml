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
        tag: LitStr,
        name: Ident,
        fields: Vec<Field>,
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
        tags: Vec<LitStr>,
        name: Ident,
        ty: Type,
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
        name: Ident,
        ty: Type,
        tag: LitStr,
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
        name: Ident,
        ty: Type,
        default: bool,
        tags: Vec<LitStr>,
    },
    /// Text Field
    ///
    /// ```ignore
    /// struct Foo {
    ///     #[xml(text, $default)]
    ///     $name: $ty,
    /// }
    /// ```
    Text { name: Ident, ty: Type },
    /// Flatten Text
    ///
    /// ```ignore
    /// struct Foo {
    ///     #[xml(flatten_text = "$tag", $default)]
    ///     $name: $ty,
    /// }
    /// ```
    FlattenText {
        name: Ident,
        ty: Type,
        default: bool,
        tag: LitStr,
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

        for meta in attrs.into_iter().filter_map(get_xml_meta).flatten() {
            match meta {
                NestedMeta::Meta(NameValue(m)) if m.path.is_ident("tag") => {
                    if let Str(lit) = m.lit {
                        tags.push(lit);
                    } else {
                        panic!("Expected a string literal.");
                    }
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
                fields: Vec::new(),
            },
            syn::Fields::Unnamed(_) => Fields::Newtype {
                name,
                tags,
                ty: Type::parse(fields.into_iter().next().unwrap().ty),
            },
            syn::Fields::Named(_) => Fields::Named {
                name,
                tag: tags.remove(0),
                fields: fields.into_iter().map(Field::parse).collect::<Vec<_>>(),
            },
        }
    }
}

impl Field {
    pub fn parse(field: syn::Field) -> Field {
        let mut default = false;
        let mut attr_tag = None;
        let mut child_tags = Vec::new();
        let mut is_text = false;
        let mut flatten_text_tag = None;

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
                        } else if !child_tags.is_empty() {
                            panic!("`attr` attribute and `child` attribute is disjoint.");
                        } else if flatten_text_tag.is_some() {
                            panic!("`attr` attribute and `flatten_text` attribute is disjoint.");
                        } else {
                            attr_tag = Some(lit);
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
                NestedMeta::Meta(NameValue(m)) if m.path.is_ident("child") => {
                    if let Str(lit) = m.lit {
                        if is_text {
                            panic!("`child` attribute and `text` attribute is disjoint.");
                        } else if attr_tag.is_some() {
                            panic!("`child` attribute and `attr` attribute is disjoint.");
                        } else if flatten_text_tag.is_some() {
                            panic!("`child` attribute and `flatten_text` attribute is disjoint.");
                        } else {
                            child_tags.push(lit);
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
                            flatten_text_tag = Some(lit);
                        }
                    } else {
                        panic!("Expected a string literal.");
                    }
                }
                _ => (),
            }
        }

        if let Some(tag) = attr_tag {
            Field::Attribute {
                name: field.ident.unwrap(),
                ty: Type::parse(field.ty),
                tag,
                default,
            }
        } else if !child_tags.is_empty() {
            Field::Child {
                name: field.ident.unwrap(),
                ty: Type::parse(field.ty),
                default,
                tags: child_tags,
            }
        } else if is_text {
            Field::Text {
                name: field.ident.unwrap(),
                ty: Type::parse(field.ty),
            }
        } else if let Some(tag) = flatten_text_tag {
            Field::FlattenText {
                name: field.ident.unwrap(),
                ty: Type::parse(field.ty),
                default,
                tag,
            }
        } else {
            panic!("Field should have one of `attr`, `child`, `text` or `flatten_text` attribute.");
        }
    }
}

impl Type {
    pub fn is_option(&self) -> bool {
        matches!(self, Type::OptionCowStr | Type::OptionT(_) | Type::OptionBool)
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
