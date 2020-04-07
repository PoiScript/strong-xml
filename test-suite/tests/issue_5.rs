use strong_xml::{XmlRead, XmlResult, XmlWrite};

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "root")]
struct Root {
    #[xml(child = "child")]
    child: Child,
}

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "child")]
struct Child {
    #[xml(text)]
    content: bool,
}

#[test]
fn test() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(
        Root {
            child: Child { content: false }
        },
        Root::from_str(r#"<root><child>false</child></root>"#)?
    );

    assert_eq!(
        Root {
            child: Child { content: false }
        },
        Root::from_str(
            r#"
<root>
<![CDATA[>]]>
<child>false</child>
<!--
    multi-line comment
-->
</root>
            "#
        )?
    );

    Ok(())
}
