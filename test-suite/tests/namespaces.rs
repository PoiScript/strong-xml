use strong_xml::{XmlRead, XmlResult, XmlWrite};

#[test]
fn test() -> XmlResult<()> {

    #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
    #[xml(tag = "nested", prefix = "n", ns = "n: http://www.example.com")]
    struct Nested {
        #[xml(child = "nested")]
        contents: Vec<Nested>,
    }


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
        r#"<n:nested xmlns:n="http://www.example.com"><n:nested><n:nested><n:nested/></n:nested></n:nested></n:nested>"#
    );
    
    Ok(())
}



#[test]
fn test2() -> XmlResult<()> {

    #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
    #[xml(tag = "nested", prefix = "n", ns = "n: http://www.example.com", ns = "n2: http://www.example.com")]
    struct Nested {
        #[xml(child="nested")]
        nested: Vec<Nested>,
        #[xml(child="nested2")]
        nested2: Nested2
    }

    #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
    #[xml(tag = "nested2", prefix = "n2",  ns = "n2: http://www.example.com")]
    struct Nested2 {
        #[xml(attr="nest", prefix = "n2")]
        value: String
    }


    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();
    
    assert_eq!(
        (Nested {
            nested: vec![],
            nested2: Nested2 { value: "hello world".into() }
        })
        .to_string()?,
        r#"<n:nested xmlns:n="http://www.example.com" xmlns:n2="http://www.example.com"><n2:nested2 n2:nest="hello world"/></n:nested>"#
    );
    
    Ok(())
}
