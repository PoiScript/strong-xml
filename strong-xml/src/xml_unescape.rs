use memchr::memchr;
use std::borrow::Cow;
use std::char;

use crate::{XmlError, XmlResult};

pub fn xml_unescape<'a>(raw: &'a str) -> XmlResult<Cow<'a, str>> {
    let bytes = raw.as_bytes();

    if let Some(i) = memchr(b'&', bytes) {
        let mut result = String::with_capacity(raw.len());

        result.push_str(&raw[0..i]);

        let mut pos = i + 1;

        if let Some(i) = memchr(b';', &bytes[pos..]) {
            recognize(&raw[pos..pos + i], &mut result)?;

            pos += i + 1;
        } else {
            return Err(XmlError::UnterminatedEntity {
                entity: String::from(&raw[pos - 1..]),
            });
        }

        while let Some(i) = memchr(b'&', &bytes[pos..]) {
            result.push_str(&raw[pos..pos + i]);

            pos += i + 1;

            if let Some(i) = memchr(b';', &bytes[pos..]) {
                recognize(&raw[pos..pos + i], &mut result)?;

                pos += i + 1;
            } else {
                return Err(XmlError::UnterminatedEntity {
                    entity: String::from(&raw[pos - 1..]),
                });
            }
        }

        result.push_str(&raw[pos..]);

        Ok(Cow::Owned(result))
    } else {
        Ok(Cow::Borrowed(raw))
    }
}

fn recognize(entity: &str, result: &mut String) -> XmlResult<()> {
    match entity {
        "quot" => result.push('"'),
        "apos" => result.push('\''),
        "gt" => result.push('>'),
        "lt" => result.push('<'),
        "amp" => result.push('&'),
        _ => {
            let val = if entity.starts_with("#x") {
                u32::from_str_radix(&entity[2..], 16).ok()
            } else if entity.starts_with('#') {
                u32::from_str_radix(&entity[1..], 10).ok()
            } else {
                None
            };
            match val.and_then(char::from_u32) {
                Some(c) => result.push(c),
                None => {
                    return Err(XmlError::UnrecognizedSymbol {
                        symbol: String::from(entity),
                    })
                }
            }
        }
    }
    Ok(())
}

#[test]
fn test_unescape() {
    assert_eq!(xml_unescape("test").unwrap(), "test");
    assert_eq!(xml_unescape("&lt;test&gt;").unwrap(), "<test>");
    assert_eq!(xml_unescape("&#x30;").unwrap(), "0");
    assert_eq!(xml_unescape("&#48;").unwrap(), "0");
}
