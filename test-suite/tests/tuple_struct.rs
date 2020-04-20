use std::borrow::Cow;
use strong_xml::{XmlRead, XmlResult, XmlWrite};

#[derive(XmlWrite, XmlRead)]
#[xml(tag = "tag1")]
struct Tag1<'a>(
    #[xml(attr = "att1")] Option<Cow<'a, str>>,
    #[xml(text)] Cow<'a, str>,
);

#[derive(XmlWrite, XmlRead)]
#[xml(tag = "tag2")]
struct Tag2<'a>(
    #[xml(attr = "att1")] Cow<'a, str>,
    #[xml(attr = "att2")] Cow<'a, str>,
);

#[derive(XmlWrite, XmlRead)]
#[xml(tag = "tag3")]
struct Tag3<'a>(
    #[xml(attr = "att1")] Cow<'a, str>,
    #[xml(child = "tag1")] Vec<Tag1<'a>>,
    #[xml(child = "tag2")] Option<Tag2<'a>>,
    #[xml(flatten_text = "text")] Option<Cow<'a, str>>,
);

#[test]
fn test() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(
        (Tag3(
            "att1".into(),
            vec![Tag1(None, "content".into())],
            None,
            Some("tag3_content".into()),
        ))
        .to_string()?,
        r#"<tag3 att1="att1"><tag1>content</tag1><text>tag3_content</text></tag3>"#
    );

    assert_eq!(
        (Tag3(
            "att1".into(),
            vec![
                Tag1(Some("att11".into()), "content1".into()),
                Tag1(Some("att12".into()), "content2".into()),
            ],
            None,
            None,
        ))
        .to_string()?,
        r#"<tag3 att1="att1"><tag1 att1="att11">content1</tag1><tag1 att1="att12">content2</tag1></tag3>"#
    );

    Ok(())
}
