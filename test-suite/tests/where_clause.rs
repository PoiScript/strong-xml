use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;

use strong_xml::{XmlRead, XmlReadOwned, XmlResult, XmlWrite};

#[derive(XmlRead, XmlWrite, PartialEq, Debug)]
#[xml(tag = "wrapper")]
struct Wrapper<T, U>
where
    T: XmlReadOwned + XmlWrite,
    U: Display + FromStr,
    // This bounds is required because we need to wrap
    // the error with a `Box<dyn Error>`
    <U as FromStr>::Err: 'static + Error + Send + Sync,
{
    #[xml(attr = "attr")]
    attr: U,
    #[xml(child = "empty")]
    child: T,
}

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "empty")]
struct Empty;

#[test]
fn test() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(
        Wrapper {
            attr: String::from("attr"),
            child: Empty
        }
        .to_string()?,
        r#"<wrapper attr="attr"><empty/></wrapper>"#
    );

    assert_eq!(
        Wrapper::from_str(r#"<wrapper attr="false"><empty/></wrapper>"#)?,
        Wrapper {
            attr: false,
            child: Empty
        }
    );

    Ok(())
}
