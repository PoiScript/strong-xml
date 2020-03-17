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
enum ABCD {
    #[xml(tag = "a", tag = "b")]
    AB(AB),
    #[xml(tag = "c", tag = "d")]
    CD(CD),
}

#[test]
fn test() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!((ABCD::AB(AB::A(A))).to_string()?, "<a/>");

    assert_eq!(ABCD::from_str("<d/>")?, ABCD::CD(CD::D(D)));

    Ok(())
}
