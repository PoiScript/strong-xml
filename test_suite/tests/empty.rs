use strong_xml::{XmlRead, XmlResult, XmlWrite};

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "empty")]
struct Empty;

#[test]
fn test() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(Empty.to_string()?, "<empty/>");

    assert_eq!(Empty::from_str("<empty/>")?, Empty);

    Ok(())
}
