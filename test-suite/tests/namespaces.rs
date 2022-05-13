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
    #[xml(
        tag = "nested",
        prefix = "n",
        ns = "n: http://www.example.com",
        ns = "n2: http://www.example.com"
    )]
    struct Nested {
        #[xml(child = "nested", prefix = "n")]
        nested: Vec<Nested>,
        #[xml(child = "nested2", prefix = "n2")]
        nested2: Nested2,
    }

    #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
    #[xml(tag = "nested2", prefix = "n2", ns = "n2: http://www.example.com")]
    struct Nested2 {
        #[xml(attr = "nest", prefix = "n2")]
        value: String,
    }

    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(
        (Nested {
            nested: vec![],
            nested2: Nested2 {
                value: "hello world".into()
            }
        })
        .to_string()?,
        r#"<n:nested xmlns:n="http://www.example.com" xmlns:n2="http://www.example.com"><n2:nested2 n2:nest="hello world"/></n:nested>"#
    );

    /*
    assert_eq!(
        (Nested {
            nested: vec![],
            nested2: Nested2 { value: "hello world".into() }
        }),
        Nested::from_str(r#"<n:nested xmlns:n="http://www.example.com" xmlns:n2="http://www.example.com"><n2:nested2 n2:nest="hello world"/></n:nested>"#)?
    );
    */

    #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
    #[xml(tag = "tag", prefix = "a", ns = "a: http://www.example.com")]
    struct A {
        #[xml(text)]
        value: String,
    }

    #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
    #[xml(tag = "tag", prefix = "b", ns = "b: http://www.example.com/1")]
    struct B {
        #[xml(text)]
        value: String,
    }

    #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
    #[xml(tag = "root", prefix = "a", ns = "a: ns_a", ns = "b: ns_b")]
    struct C {
        #[xml(child = "tag", prefix = "a")]
        a: A,
        #[xml(child = "tag", prefix = "b")]
        b: B,
    }

    assert_eq!(
        C {
            a: A {
                value: "hello".into()
            },
            b: B {
                value: "world".into()
            }
        },
        C::from_str(
            r#"<a:root xmlns:a="ns_a" xmlns:b="ns_b"><a:tag>hello</a:tag><b:tag>world</b:tag></a:root>"#
        )?
    );

    Ok(())
}
