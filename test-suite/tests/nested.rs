use strong_xml::{XmlRead, XmlResult, XmlWrite};

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "nested")]
struct Nested {
    #[xml(child = "nested")]
    contents: Vec<Nested>,
}

#[test]
fn test() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(
        (Nested {
            contents: vec![Nested {
                contents: vec![Nested {
                    contents: vec![Nested { contents: vec![] }]
                }]
            }]
        })
        .to_string()?,
        r#"<nested><nested><nested><nested/></nested></nested></nested>"#
    );

    Ok(())
}
