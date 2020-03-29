use std::str::FromStr;

use strong_xml::{XmlRead, XmlResult};

#[derive(PartialEq, Debug)]
struct Foo;

impl FromStr for Foo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "foo" || s == "FOO" {
            Ok(Foo)
        } else {
            Err("invalid Foo".into())
        }
    }
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "root")]
struct Root {
    #[xml(attr = "foo")]
    foo: Foo,
    #[xml(attr = "bar")]
    bar: Option<String>,
    #[xml(attr = "baz")]
    baz: Option<usize>,
}

#[test]
fn test() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(
        Root::from_str(r#"<root foo="foo" baz="100"/>"#)?,
        Root {
            foo: Foo,
            bar: None,
            baz: Some(100)
        }
    );

    assert_eq!(
        Root::from_str(r#"<root foo="FOO" bar="bar"/>"#)?,
        Root {
            foo: Foo,
            bar: Some("bar".into()),
            baz: None
        }
    );

    assert!(Root::from_str(r#"<root foo="bar"/>"#).is_err());

    assert!(Root::from_str(r#"<root foo="foo" baz="baz"/>"#).is_err());

    Ok(())
}
