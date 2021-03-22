use std::borrow::Cow;
use strong_xml::{XmlRead, XmlResult, XmlWrite};

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "root")]
struct Root<'a> {
    #[xml(text)]
    content: Cow<'a, str>,
}

#[test]
fn test() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(
        Root { content: "".into() },
        Root::from_str(r#"<root></root>"#)?
    );

    assert_eq!(
        Root {
            content: "content".into()
        },
        Root::from_str(r#"<root>content</root>"#)?
    );

    assert_eq!(
        Root { content: "".into() },
        Root::from_str(r#"<root><![CDATA[]]></root>"#)?
    );

    assert_eq!(
        Root {
            content: "&lt;&gt;".into()
        },
        Root::from_str(r#"<root><![CDATA[&lt;&gt;]]></root>"#)?
    );

    assert_eq!(
        Root {
            content: "</>".into()
        },
        Root::from_str(r#"<root><![CDATA[</>]]></root>"#)?
    );

    Ok(())
}
