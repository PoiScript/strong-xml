use strong_xml::{XmlRead, XmlResult, XmlWrite};

#[derive(Debug, PartialEq, XmlRead, XmlWrite)]
#[xml(tag = "head")]
struct Head {
    #[xml(flatten_text = "title")]
    title: String,
}

#[test]
fn test() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(
        Head::from_str(r#"<head><title/></head>"#)?,
        Head {
            title: "".into(),
        }
    );

    assert_eq!(
        Head {
            title: "".into(),
        }
        .to_string()?,
        r#"<head><title></title></head>"#
    );

    Ok(())
}
