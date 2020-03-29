use std::fmt;

use strong_xml::{XmlResult, XmlWrite};

#[derive(PartialEq, Debug)]
struct Foo;

impl fmt::Display for Foo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "foo")
    }
}

#[derive(XmlWrite, PartialEq, Debug)]
#[xml(tag = "root")]
struct Root {
    #[xml(attr = "foo")]
    foo: Option<Foo>,
    #[xml(attr = "bar")]
    bar: String,
}

#[test]
fn test() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(
        (Root {
            foo: None,
            bar: "bar".into()
        })
        .to_string()?,
        r#"<root bar="bar"/>"#
    );

    assert_eq!(
        (Root {
            foo: Some(Foo),
            bar: "bar".into()
        })
        .to_string()?,
        r#"<root foo="foo" bar="bar"/>"#
    );

    Ok(())
}
