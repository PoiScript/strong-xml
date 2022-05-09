use strong_xml::{XmlRead, XmlResult, XmlWrite};

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "nested", prefix = "n", ns = "n: http://www.example.com")]
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
        r#"<n:nested><n:nested><n:nested><n:nested/></n:nested></n:nested></n:nested>"#
    );
    
    Ok(())
}
