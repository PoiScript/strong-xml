use chrono::offset::TimeZone;
use chrono::{DateTime, Utc};
use strong_xml::{XmlRead, XmlResult, XmlWrite};

#[derive(Debug, PartialEq, XmlRead, XmlWrite)]
#[xml(tag = "document")]
struct Document {
    #[xml(attr = "datetime")]
    datetime: DateTime<Utc>,
}

#[test]
fn test() -> XmlResult<()> {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    assert_eq!(
        Document::from_str(r#"<document datetime="1970-01-01T00:00:00.0Z" />"#)?,
        Document {
            datetime: Utc.timestamp(0, 0)
        }
    );

    assert_eq!(
        (Document {
            datetime: Utc.ymd(2018, 1, 26).and_hms_micro(18, 30, 9, 453_829)
        })
        .to_string()?,
        r#"<document datetime="2018-01-26 18:30:09.453829 UTC"/>"#
    );

    Ok(())
}
