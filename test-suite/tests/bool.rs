use strong_xml::{XmlRead, XmlResult, XmlWrite};

#[derive(XmlRead, XmlWrite, PartialEq, Debug)]
#[xml(tag = "attr")]
struct Attr {
    #[xml(attr = "attr")]
    attr: Option<bool>,
}


#[derive(XmlRead, XmlWrite, PartialEq, Debug)]
#[xml(tag = "text")]
struct FlattenText {
    #[xml(flatten_text = "foo")]
    foo: Vec<bool>,
}

#[derive(XmlRead, XmlWrite, PartialEq, Debug)]
#[xml(tag = "text")]
struct Text {
    #[xml(text)]
    text: bool,
}

#[test]
fn test() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(
        Attr::from_str(r#"<attr foo="bar" />"#)?,
        Attr { attr: None }
    );

    assert_eq!(
        Attr::from_str(r#"<attr attr="1" />"#)?,
        Attr { attr: Some(true) }
    );

    assert_eq!(
        (Attr { attr: Some(true) }).to_string()?,
        r#"<attr attr="true"/>"#,
    );

    assert_eq!(Text::from_str(r#"<text>off</text>"#)?, Text { text: false });

    assert_eq!((Text { text: false }).to_string()?, r#"<text>false</text>"#);

    assert_eq!(
        FlattenText::from_str(r#"<text></text>"#)?,
        FlattenText { foo: vec![] }
    );

    assert_eq!(
        FlattenText::from_str(r#"<text><foo>1</foo></text>"#)?,
        FlattenText { foo: vec![true] }
    );

    assert_eq!(
        FlattenText::from_str(r#"<text><foo>f</foo><foo>t</foo></text>"#)?,
        FlattenText {
            foo: vec![false, true]
        }
    );

    assert_eq!(
        (FlattenText {
            foo: vec![false, true]
        })
        .to_string()?,
        r#"<text><foo>false</foo><foo>true</foo></text>"#,
    );

    Ok(())
}
