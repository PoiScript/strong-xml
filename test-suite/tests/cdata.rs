use strong_xml::{XmlRead, XmlResult, XmlWrite};
#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "bar")]
struct Bar(#[xml(text, cdata)] String);

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "foo")]
struct Foo {
    #[xml(child = "bar")]
    bar: Bar,

    #[xml(flatten_text = "qux", cdata)]
    baz: String,
}

#[test]
fn test() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(
        Foo {
            bar: Bar("<".into()),
            baz: ">".into()
        },
        Foo::from_str(r#"<foo><bar>&lt;</bar><qux>&gt;</qux></foo>"#)?
    );

    assert_eq!(
        Foo {
            bar: Bar("<".into()),
            baz: ">".into()
        },
        Foo::from_str(r#"<foo><bar><![CDATA[<]]></bar><qux><![CDATA[>]]></qux></foo>"#)?
    );

    assert_eq!(
        Foo {
            bar: Bar("&lt;".into()),
            baz: "&gt;".into()
        },
        Foo::from_str(r#"<foo><bar><![CDATA[&lt;]]></bar><qux><![CDATA[&gt;]]></qux></foo>"#)?
    );

    assert_eq!(
        (Foo {
            bar: Bar("&lt;".into()),
            baz: "&gt;".into()
        })
        .to_string()?,
        r#"<foo><bar><![CDATA[&lt;]]></bar><qux><![CDATA[&gt;]]></qux></foo>"#,
    );

    assert_eq!(
        (Foo {
            bar: Bar("<".into()),
            baz: ">".into()
        })
        .to_string()?,
        r#"<foo><bar><![CDATA[<]]></bar><qux><![CDATA[>]]></qux></foo>"#,
    );

    Ok(())
}
