use std::borrow::Cow;
use strong_xml::{XmlRead, XmlResult, XmlWrite};

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "root")]
struct Root<'a> {
    #[xml(text)]
    content: Cow<'a, str>,
}

#[derive(XmlWrite, PartialEq, Debug)] // XmlRead
#[xml(tag = "root")]
struct TypedRoot {
    #[xml(text)]
    id: usize,
}

#[derive(XmlWrite, PartialEq, Debug)] // XmlRead
#[xml(tag = "root")]
struct Test {
    #[xml(flatten_text = "my:confirm")]
    confirm: bool,
}

#[test]
fn test() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(Root { content: "".into() }, Root::from_str(r#"<root/>"#)?);

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
        r#"<root>42</root>"#,
        TypedRoot { id: 42 }.to_string().unwrap()
    );
    // assert_eq!(
    //     TypedRoot { id: 42 },
    //     TypedRoot::from_str(r#"<root>42</root>"#)?
    // );

    assert_eq!(
        r#"<root><my:confirm>true</my:confirm></root>"#,
        Test { confirm: true }.to_string().unwrap()
    );
    // assert_eq!(
    //     Test { confirm: true },
    //     Test::from_str(r#"<root><my:confirm>true</my:confirm></root>"#)?
    // );

    Ok(())
}
