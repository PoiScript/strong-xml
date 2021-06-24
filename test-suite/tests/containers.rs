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
    #[xml(child = "tag1", container = "container", default)]
    tag1: Vec<Tag1<'a>>,
}

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "tag2")]
struct Tag2Opt<'a> {
    #[xml(child = "tag1", container = "container")]
    tag1: Option<Vec<Tag1<'a>>>,
}

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "tag3")]
struct Tag3 {
    #[xml(flatten_text = "value", container = "container", default)]
    data: Vec<u64>,
}

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "tag3")]
struct Tag3Opt {
    #[xml(flatten_text = "value", container = "container")]
    data: Option<Vec<u64>>,
}

#[test]
fn test() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(
        Tag2::from_str(
            r#"<tag2><container><tag1 att1="att1">foo</tag1><tag1>bar</tag1></container></tag2>"#
        )?,
        Tag2 {
            tag1: vec![
                Tag1 {
                    att1: Some("att1".into()),
                    content: "foo".into(),
                },
                Tag1 {
                    att1: None,
                    content: "bar".into(),
                }
            ],
        }
    );

    assert_eq!(
        Tag3::from_str(
            r#"<tag3><container><value>123</value><value>0</value></container></tag3>"#
        )?,
        Tag3 { data: vec![123, 0] }
    );

    assert_eq!(
        Tag2 {
            tag1: vec![
                Tag1 {
                    att1: Some("att1".into()),
                    content: "foo".into(),
                },
                Tag1 {
                    att1: None,
                    content: "bar".into(),
                }
            ],
        }
        .to_string()?,
        r#"<tag2><container><tag1 att1="att1">foo</tag1><tag1>bar</tag1></container></tag2>"#,
    );

    assert_eq!(
        Tag3 { data: vec![123, 0] }.to_string()?,
        r#"<tag3><container><value>123</value><value>0</value></container></tag3>"#,
    );

    Ok(())
}

#[test]
fn test_empty() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(Tag2::from_str(r#"<tag2></tag2>"#)?, Tag2 { tag1: vec![] });

    assert_eq!(
        Tag2::from_str(r#"<tag2><container></container></tag2>"#)?,
        Tag2 { tag1: vec![] }
    );

    assert_eq!(
        Tag2::from_str(r#"<tag2><container><unknown/></container></tag2>"#)?,
        Tag2 { tag1: vec![] }
    );

    assert_eq!(
        Tag2Opt::from_str(r#"<tag2></tag2>"#)?,
        Tag2Opt { tag1: None }
    );

    assert_eq!(
        Tag2Opt::from_str(r#"<tag2><container></container></tag2>"#)?,
        Tag2Opt { tag1: Some(vec![]) }
    );

    assert_eq!(
        Tag2Opt::from_str(r#"<tag2><container><unknown/></container></tag2>"#)?,
        Tag2Opt { tag1: Some(vec![]) }
    );

    assert_eq!(Tag3::from_str(r#"<tag3></tag3>"#)?, Tag3 { data: vec![] });

    assert_eq!(
        Tag3::from_str(r#"<tag3><container></container></tag3>"#)?,
        Tag3 { data: vec![] }
    );

    assert_eq!(
        Tag3::from_str(r#"<tag3><container><unknown/></container></tag3>"#)?,
        Tag3 { data: vec![] }
    );

    assert_eq!(
        Tag3Opt::from_str(r#"<tag3></tag3>"#)?,
        Tag3Opt { data: None }
    );

    assert_eq!(
        Tag3Opt::from_str(r#"<tag3><container></container></tag3>"#)?,
        Tag3Opt { data: Some(vec![]) }
    );

    assert_eq!(
        Tag3Opt::from_str(r#"<tag3><container><unknown/></container></tag3>"#)?,
        Tag3Opt { data: Some(vec![]) }
    );

    Ok(())
}
