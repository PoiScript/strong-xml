use std::borrow::Cow;
use strong_xml::{XmlRead, XmlResult, XmlWrite};

#[derive(Default, XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "root")]
struct Root<'a> {
    #[xml(default, attr = "att1")]
    att1: Cow<'a, str>,
    #[xml(default, child = "child")]
    child: Child<'a>,
}

#[derive(Default, XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "child")]
struct Child<'a> {
    #[xml(default, attr = "att1")]
    att1: Cow<'a, str>,
    #[xml(default, attr = "att2")]
    att2: bool,
}

#[test]
fn test() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(Root::default(), Root::from_str(r#"<root/>"#)?);

    assert_eq!(
        Root {
            att1: "value".into(),
            child: Child::default()
        },
        Root::from_str(r#"<root att1="value"></root>"#)?
    );

    assert_eq!(
        Root {
            att1: "value".into(),
            child: Child::default()
        },
        Root::from_str(r#"<root att1="value"><child></child></root>"#)?
    );

    assert_eq!(
        Root {
            att1: "value".into(),
            child: Child {
                att1: "value".into(),
                ..Default::default()
            }
        },
        Root::from_str(r#"<root att1="value"><child att1="value"></child></root>"#)?
    );

    assert_eq!(
        Root {
            att1: "value".into(),
            child: Child {
                att2: true,
                ..Default::default()
            }
        },
        Root::from_str(r#"<root att1="value"><child att2="true"></child></root>"#)?
    );

    Ok(())
}
