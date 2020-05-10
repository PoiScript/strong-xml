use std::borrow::Cow;
use strong_xml::{XmlRead, XmlResult, XmlWrite};

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "tag1")]
struct Tag1<'a> {
    #[xml(attr = "att1")]
    att1: Option<Cow<'a, str>>,
    #[xml(text)]
    content: Cow<'a, str>,
}

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "tag2")]
struct Tag2<'a> {
    #[xml(attr = "att1")]
    att1: Cow<'a, str>,
    #[xml(attr = "att2")]
    att2: Cow<'a, str>,
}

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "tag3")]
struct Tag3<'a> {
    #[xml(attr = "att1")]
    att1: Cow<'a, str>,
    #[xml(child = "tag1")]
    tag1: Vec<Tag1<'a>>,
    #[xml(child = "tag2")]
    tag2: Option<Tag2<'a>>,
    #[xml(flatten_text = "text")]
    text: Option<Cow<'a, str>>,
}

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "tag3")]
struct Tag4<'a>(Tag3<'a>);

#[test]
fn test() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(
        (Tag3 {
            att1: "att1".into(),
            tag1: vec![Tag1 {
                att1: None,
                content: "content".into(),
            }],
            tag2: None,
            text: Some("tag3_content".into()),
        })
        .to_string()?,
        r#"<tag3 att1="att1"><tag1>content</tag1><text>tag3_content</text></tag3>"#
    );

    assert_eq!(
        (Tag3 {
            att1: "att1".into(),
            tag1: vec![
                Tag1 {
                    att1: Some("att11".into()),
                    content: "content1".into(),
                },
                Tag1 {
                    att1: Some("att12".into()),
                    content: "content2".into(),
                },
            ],
            tag2: None,
            text: None,
        })
        .to_string()?,
        r#"<tag3 att1="att1"><tag1 att1="att11">content1</tag1><tag1 att1="att12">content2</tag1></tag3>"#
    );

    assert_eq!(
        Tag4::from_str(r#"<tag3 att1="att1"><tag2 att1="att1" att2="att2"></tag2></tag3>"#)?,
        Tag4(Tag3 {
            att1: "att1".into(),
            tag1: vec![],
            tag2: Some(Tag2 {
                att1: "att1".into(),
                att2: "att2".into(),
            }),
            text: None,
        })
    );

    Ok(())
}
