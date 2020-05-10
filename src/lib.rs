//! # deprecated
//!
//! `xmlparser-dervie` has been renamed to `strong-xml` since v0.1.2.
//!
//! A proc macro to generate functions for writing to
//! and parsing from xml string, based on xmlparser.
//!
//! ## Quick Start
//!
//! ```toml
//! xmlparser = "0.13.0"
//! xmlparser-derive = "0.1.0"
//! ```
//!
//! ```rust
//! use std::borrow::Cow;
//! use xmlparser_derive::{XmlRead, XmlWrite};
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "parent")]
//! struct Parent<'a> {
//!     #[xml(attr = "attr1")]
//!     attr1: Cow<'a, str>,
//!     #[xml(attr = "attr2")]
//!     attr2: Option<Cow<'a, str>>,
//!     #[xml(child = "child")]
//!     child: Vec<Child<'a>>,
//! }
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "child")]
//! struct Child<'a> {
//!     #[xml(text)]
//!     text: Cow<'a, str>,
//! }
//!
//! assert_eq!(
//!     (Parent { attr1: "val".into(), attr2: None, child: vec![] }).to_string().unwrap(),
//!     r#"<parent attr1="val"></parent>"#
//! );
//!
//! assert_eq!(
//!     Parent::from_str(r#"<parent attr1="val" attr2="val"><child></child></parent>"#).unwrap(),
//!     Parent { attr1: "val".into(), attr2: Some("val".into()), child: vec![Child { text: "".into() }] }
//! );
//! ```
//!
//! ## Attributes
//!
//! ### `#[xml(tag = "")]`
//!
//! Specifies the xml tag of a struct or an enum variant.
//!
//! ```rust
//! # use xmlparser_derive::{XmlRead, XmlWrite};
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "parent")]
//! struct Parent {}
//!
//! assert_eq!(
//!     (Parent {}).to_string().unwrap(),
//!     r#"<parent></parent>"#
//! );
//!
//! assert_eq!(
//!     Parent::from_str(r#"<parent></parent>"#).unwrap(),
//!     Parent {}
//! );
//! ```
//!
//! ```rust
//! # use xmlparser_derive::{XmlRead, XmlWrite};
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "tag1")]
//! struct Tag1 {}
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "tag2")]
//! struct Tag2 {}
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! enum Tag {
//!     #[xml(tag = "tag1")]
//!     Tag1(Tag1),
//!     #[xml(tag = "tag2")]
//!     Tag2(Tag2),
//! }
//!
//! assert_eq!(
//!     (Tag::Tag1(Tag1 {})).to_string().unwrap(),
//!     r#"<tag1></tag1>"#
//! );
//!
//! assert_eq!(
//!     Tag::from_str(r#"<tag2></tag2>"#).unwrap(),
//!     Tag::Tag2(Tag2 {})
//! );
//! ```
//!
//! ### `#[xml(attr = "")]`
//!
//! Specifies that a struct field is attribute. Support
//! `Cow<str>`, `Option<Cow<str>>`, `bool`, `Option<bool>`,
//! `usize`, `Option<usize>`, `T` and `Option<T>` where
//! `T: std::str::FromStr`.
//!
//! ```rust
//! use xmlparser_derive::{XmlRead, XmlWrite};
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "parent")]
//! struct Parent {
//!     #[xml(attr = "attr")]
//!     attr: usize
//! }
//!
//! assert_eq!(
//!     (Parent { attr: 42 }).to_string().unwrap(),
//!     r#"<parent attr="42"></parent>"#
//! );
//!
//! assert_eq!(
//!     Parent::from_str(r#"<parent attr="48"></parent>"#).unwrap(),
//!     Parent { attr: 48 }
//! );
//! ```
//!
//! ### `#[xml(leaf)]`
//!
//! Specifies that a sturct is leaf element.
//!
//! ```rust
//! # use xmlparser_derive::{XmlRead, XmlWrite};
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(leaf, tag = "leaf")]
//! struct Leaf {}
//!
//! assert_eq!(
//!     (Leaf {}).to_string().unwrap(),
//!     r#"<leaf/>"#
//! );
//!
//! assert_eq!(
//!     Leaf::from_str(r#"<leaf/>"#).unwrap(),
//!     Leaf {}
//! );
//! ```
//!
//! ### `#[xml(child = "")]`
//!
//! Specifies that a struct field is a child element. Support
//! `T`, `Option<T>`, `Vec<T>` where `T: XmlRead + XmlWrite`.
//!
//! ```rust
//! use xmlparser_derive::{XmlRead, XmlWrite};
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "tag1")]
//! struct Tag1 {}
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "tag2")]
//! struct Tag2 {}
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "tag3")]
//! struct Tag3 {}
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! enum Tag12 {
//!     #[xml(tag = "tag1")]
//!     Tag1(Tag1),
//!     #[xml(tag = "tag2")]
//!     Tag2(Tag2),
//! }
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "parent")]
//! struct Parent {
//!     #[xml(child = "tag3")]
//!     tag3: Vec<Tag3>,
//!     #[xml(child = "tag1", child = "tag2")]
//!     tag12: Option<Tag12>
//! }
//!
//! assert_eq!(
//!     (Parent { tag3: vec![Tag3 {}], tag12: None }).to_string().unwrap(),
//!     r#"<parent><tag3></tag3></parent>"#
//! );
//!
//! assert_eq!(
//!     Parent::from_str(r#"<parent><tag2></tag2></parent>"#).unwrap(),
//!     Parent { tag3: vec![], tag12: Some(Tag12::Tag2(Tag2 {})) }
//! );
//! ```
//!
//! ### `#[xml(text)]`
//!
//! Specifies that a struct field is text content.
//! Support `Cow<str>`.
//!
//! ```rust
//! use std::borrow::Cow;
//! use xmlparser_derive::{XmlRead, XmlWrite};
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "parent")]
//! struct Parent<'a> {
//!     #[xml(text)]
//!     content: Cow<'a, str>,
//! }
//!
//! assert_eq!(
//!     (Parent { content: "content".into() }).to_string().unwrap(),
//!     r#"<parent>content</parent>"#
//! );
//!
//! assert_eq!(
//!     Parent::from_str(r#"<parent></parent>"#).unwrap(),
//!     Parent { content: "".into() }
//! );
//! ```
//!
//! ### `#[xml(flatten_text = "")]`
//!
//! Specifies that a struct field is child text element.
//! Support `Cow<str>`, `Vec<Cow<str>>` and `Option<Cow<str>>`.
//!
//! ```rust
//! use std::borrow::Cow;
//! use xmlparser_derive::{XmlRead, XmlWrite};
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "parent")]
//! struct Parent<'a> {
//!     #[xml(flatten_text = "child")]
//!     content: Cow<'a, str>,
//! }
//!
//! assert_eq!(
//!     (Parent { content: "content".into() }).to_string().unwrap(),
//!     r#"<parent><child>content</child></parent>"#
//! );
//!
//! assert_eq!(
//!     Parent::from_str(r#"<parent><child></child></parent>"#).unwrap(),
//!     Parent { content: "".into() }
//! );
//! ```
//!
//! ## License
//!
//! MIT

pub use xmlparser_derive_core::{XmlRead, XmlWrite};
pub use xmlparser_derive_utils::{XmlError, XmlReader, XmlResult};

pub mod utils {
    pub use xmlparser_derive_utils::{
        read_text, read_till_element_start, read_to_end, xml_escape, xml_unescape,
    };
}
