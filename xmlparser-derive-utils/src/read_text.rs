use std::borrow::Cow;
use xmlparser::{ElementEnd, Token};

use crate::{xml_unescape, XmlError, XmlReader, XmlResult};

pub fn read_text<'a>(reader: &mut XmlReader<'a>, tag: &'a str) -> XmlResult<Cow<'a, str>> {
    let mut res = None;

    while let Some(token) = reader.next() {
        match token? {
            Token::ElementEnd {
                end: ElementEnd::Open,
                ..
            }
            | Token::Attribute { .. } => (),
            Token::Text { text } => {
                res = Some(xml_unescape(text.as_str())?);
            }
            Token::ElementEnd {
                end: ElementEnd::Close(_, _),
                span,
            } => {
                let span = span.as_str();
                if tag == &span[2..span.len() - 1] {
                    break;
                } else {
                    return Err(XmlError::TagMismatch {
                        expected: tag.to_owned(),
                        found: span[2..span.len() - 1].to_owned(),
                    });
                }
            }
            token => {
                return Err(XmlError::UnexpectedToken {
                    token: format!("{:?}", token),
                });
            }
        }
    }

    Ok(res.unwrap_or_default())
}

#[test]
fn test_read_text() -> XmlResult<()> {
    use xmlparser::Tokenizer;

    let mut reader = Tokenizer::from("<parent></parent>").peekable();

    assert!(reader.next().is_some()); // "<parent"
    assert_eq!(read_text(&mut reader, "parent")?, "");
    assert!(reader.next().is_none());

    let mut reader = Tokenizer::from("<parent>text</parent>").peekable();

    assert!(reader.next().is_some()); // "<parent"
    assert_eq!(read_text(&mut reader, "parent")?, "text");
    assert!(reader.next().is_none());

    let mut reader = Tokenizer::from("<parent attr=\"value\">text</parent>").peekable();

    assert!(reader.next().is_some()); // "<parent"
    assert_eq!(read_text(&mut reader, "parent")?, "text");
    assert!(reader.next().is_none());

    Ok(())
}
