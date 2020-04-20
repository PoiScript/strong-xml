use std::borrow::Cow;
use std::fmt::Write;
use std::sync::{Arc, Mutex};

use log::{LevelFilter, Log, Metadata, Record};
use strong_xml::{XmlRead, XmlResult, XmlWrite};

struct Logger(Arc<Mutex<String>>);

impl Log for Logger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        writeln!(
            *self.0.lock().unwrap(),
            "{:<5} {}",
            record.level(),
            record.args()
        )
        .unwrap();
    }

    fn flush(&self) {}
}

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "tag1")]
struct Tag1<'a> {
    #[xml(attr = "att1")]
    att1: Option<Cow<'a, str>>,
    #[xml(text)]
    content: Cow<'a, str>,
}

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "tag2")]
struct Tag2<'a> {
    #[xml(attr = "att1")]
    att1: Cow<'a, str>,
    #[xml(attr = "att2")]
    att2: Cow<'a, str>,
}

#[derive(XmlWrite, XmlRead, PartialEq, Debug)]
#[xml(tag = "tag3")]
struct Tag3<'a> {
    #[xml(attr = "att1")]
    att1: Cow<'a, str>,
    #[xml(child = "tag1")]
    tag1: Vec<Tag1<'a>>,
    #[xml(child = "tag2")]
    tag2: Option<Tag2<'a>>,
    #[xml(flatten_text = "text")]
    text: Option<Cow<'a, str>>,
}

#[test]
fn info() -> XmlResult<()> {
    let buf = Arc::new(Mutex::new(String::new()));

    let logger = Logger(buf.clone());

    log::set_boxed_logger(Box::new(logger)).unwrap();
    log::set_max_level(LevelFilter::Trace);

    let _ = Tag3::from_str(
        "<tag3 att1=\"att1\" att2=\"att2\">\
        <tag2 att1=\"att1\" att2=\"att2\"/>\
        <tag4></tag4>\
        <tag1>content</tag1>\
        <text>tag3_content</text>\
        <tag5><tag6/></tag5>\
        </tag3>",
    )
    .unwrap();

    assert_eq!(
        r#"DEBUG [Tag3] Started reading
TRACE [Tag3] Started reading field `att1`
TRACE [Tag3] Finished reading field `att1`
INFO  [Tag3] Skip attribute `att2`
TRACE [Tag3] Started reading field `tag2`
DEBUG [Tag2] Started reading
TRACE [Tag2] Started reading field `att1`
TRACE [Tag2] Finished reading field `att1`
TRACE [Tag2] Started reading field `att2`
TRACE [Tag2] Finished reading field `att2`
DEBUG [Tag2] Finished reading
TRACE [Tag3] Finished reading field `tag2`
INFO  [Tag3] Skip element `tag4`
TRACE [Tag3] Started reading field `tag1`
DEBUG [Tag1] Started reading
TRACE [Tag1] Started reading field `content`
TRACE [Tag1] Finished reading field `content`
DEBUG [Tag1] Finished reading
TRACE [Tag3] Finished reading field `tag1`
TRACE [Tag3] Started reading field `text`
TRACE [Tag3] Finished reading field `text`
INFO  [Tag3] Skip element `tag5`
DEBUG [Tag3] Finished reading
"#,
        buf.lock().unwrap().as_str()
    );

    Ok(())
}
