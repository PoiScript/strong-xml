use strong_xml::{XmlRead, XmlResult, XmlWrite};

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "a")]
struct A;

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "b")]
struct B;

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "c")]
struct C;

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "d")]
struct D;

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
enum AB {
    #[xml(tag = "a")]
    A(A),
    #[xml(tag = "b")]
    B(B),
}

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
enum CD {
    #[xml(tag = "c")]
    C(C),
    #[xml(tag = "d")]
    D(D),
}

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
enum ABCDEFG {
    #[xml(tag = "a", tag = "b")]
    AB(AB),
    #[xml(tag = "c", tag = "d")]
    CD(CD),
    #[xml(tag = "e")]
    E,
    #[xml(tag = "f")]
    F {
        #[xml(text)]
        foo: String,
    },
    #[xml(tag = "g")]
    G {
        #[xml(attr = "foo")]
        foo: usize,
        #[xml(flatten_text = "bar")]
        bar: bool,
    },
}

#[test]
fn test() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!((ABCDEFG::AB(AB::A(A))).to_string()?, "<a/>");

    assert_eq!(
        (ABCDEFG::G {
            foo: 42,
            bar: false
        })
        .to_string()?,
        r#"<g foo="42"><bar>false</bar></g>"#
    );

    assert_eq!(ABCDEFG::from_str("<d/>")?, ABCDEFG::CD(CD::D(D)));

    assert_eq!(
        ABCDEFG::from_str("<f>foo</f>")?,
        ABCDEFG::F { foo: "foo".into() }
    );

    Ok(())
}
