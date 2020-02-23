use xmlparser::Token;

use crate::read_to_end::read_to_end;
use crate::{XmlError, XmlReader, XmlResult};

pub fn read_till_element_start(reader: &mut XmlReader<'_>, tag: &str) -> XmlResult<()> {
    while let Some(token) = reader.next() {
        let token = token?;
        match token {
            Token::ElementStart { span, .. } => {
                if tag == &span.as_str()[1..] {
                    break;
                } else {
                    // log::info!(
                    //     "Unhandled tag: {:?} when reading ParentElement {}. Skipping.",
                    //     __tag, stringify!(#ele_name)
                    // );
                    read_to_end(reader, &span.as_str()[1..])?;
                }
            }
            Token::ElementEnd { .. }
            | Token::Attribute { .. }
            | Token::Text { .. }
            | Token::Cdata { .. } => {
                return Err(XmlError::UnexpectedToken {
                    token: format!("{:?}", token),
                });
            }
            _ => (),
        }
    }

    Ok(())
}

#[test]
fn test_read_till_element_start() -> XmlResult<()> {
    use xmlparser::Tokenizer;

    let mut reader = Tokenizer::from("<tag/>").peekable();

    read_till_element_start(&mut reader, "tag")?;
    assert!(reader.next().is_some()); // "/>"
    assert!(reader.next().is_none());

    let mut reader = Tokenizer::from("<parent><skip/><tag/></parent>").peekable();

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    read_till_element_start(&mut reader, "tag")?;
    assert!(reader.next().is_some()); // "/>"
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    let mut reader = Tokenizer::from("<parent><skip></skip><tag/></parent>").peekable();

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    read_till_element_start(&mut reader, "tag")?;
    assert!(reader.next().is_some()); // "/>"
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    let mut reader = Tokenizer::from("<parent><skip><skip/></skip><tag/></parent>").peekable();

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    read_till_element_start(&mut reader, "tag")?;
    assert!(reader.next().is_some()); // "/>"
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    let mut reader =
        Tokenizer::from("<parent><skip><skip></skip></skip><tag/></parent>").peekable();

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    read_till_element_start(&mut reader, "tag")?;
    assert!(reader.next().is_some()); // "/>"
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    Ok(())
}
