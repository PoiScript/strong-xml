use strong_xml::{XmlRead, XmlResult, XmlWrite};

#[test]
fn test() -> XmlResult<()> {
    #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
    #[xml(tag = "n:nested", ns = "n: http://www.example.com")]
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
        tag = "n:nested",
        ns = "n: http://www.example.com",
        ns = "n2: http://www.example.com"
    )]
    struct Nested {
        #[xml(child = "n:nested")]
        nested: Vec<Nested>,
        #[xml(child = "n2:nested2")]
        nested2: Nested2,
    }

    #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
    #[xml(tag = "n2:nested2", ns = "n2: http://www.example.com")]
    struct Nested2 {
        #[xml(attr = "n2:nest")]
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

    assert_eq!(
        (Nested {
            nested: vec![],
            nested2: Nested2 {
                value: "hello world".into()
            }
        }),
        Nested::from_str(
            r#"<n:nested xmlns:n="http://www.example.com" xmlns:n2="http://www.example.com"><n2:nested2 n2:nest="hello world"/></n:nested>"#
        )?
    );

    #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
    #[xml(tag = "a:tag", ns = "a: http://www.example.com")]
    struct A {
        #[xml(text)]
        value: String,
    }

    #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
    #[xml(tag = "b:tag", ns = "b: http://www.example.com/1")]
    struct B {
        #[xml(text)]
        value: String,
    }

    #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
    #[xml(tag = "a:root", ns = "a: ns_a", ns = "b: ns_b")]
    struct C {
        #[xml(child = "a:tag")]
        a: A,
        #[xml(child = "b:tag")]
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
        }
        .to_string()?,
        r#"<a:root xmlns:a="ns_a" xmlns:b="ns_b"><a:tag xmlns:a="http://www.example.com">hello</a:tag><b:tag xmlns:b="http://www.example.com/1">world</b:tag></a:root>"#
    );

    /*
    TODO: namespaces are not enforced when readeing.
    Should we have options that the user can set?
    */
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
            r#"<a:root xmlns:a="ns_a" xmlns:b="ns_b"><a:tag xmlns:a="http://www.example.com">hello</a:tag><b:tag>world</b:tag></a:root>"#
        )?
    );

    Ok(())
}
