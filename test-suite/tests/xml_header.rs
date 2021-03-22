use strong_xml::{XmlRead, XmlResult, XmlWrite};

#[derive(XmlRead, XmlWrite, PartialEq, Debug)]
#[xml(tag = "foo")]
struct Foo(#[xml(text)] String);

#[test]
fn test() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(
        Foo::from_str(
            r#"<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE Server SYSTEM "opt/pdos/etc/pdoslrd.dtd"><foo>foo</foo>"#
        )?,
        Foo("foo".into())
    );

    Ok(())
}
