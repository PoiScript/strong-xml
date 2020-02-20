use std::iter::Peekable;
use xmlparser::{ElementEnd, Token, Tokenizer};

use crate::{XmlError, XmlResult};

pub fn read_to_end(reader: &mut Peekable<Tokenizer<'_>>, tag: &str) -> XmlResult<()> {
    while let Some(token) = reader.next() {
        match token? {
            // if this element is emtpy, just return
            Token::ElementEnd {
                end: ElementEnd::Empty,
                ..
            } => return Ok(()),
            Token::ElementEnd {
                end: ElementEnd::Open,
                ..
            } => break,
            Token::Attribute { .. } => (),
            // there shouldn't have any token but Attribute between ElementStart and ElementEnd
            token => {
                return Err(XmlError::UnexpectedToken {
                    token: format!("{:?}", token),
                })
            }
        }
    }

    let mut depth = 1;

    while let Some(token) = reader.next() {
        match token? {
            Token::ElementStart { span, .. } if tag == &span.as_str()[1..] => {
                while let Some(token) = reader.next() {
                    match token? {
                        Token::ElementEnd {
                            end: ElementEnd::Empty,
                            ..
                        } => {
                            if depth == 0 {
                                return Ok(());
                            } else {
                                // don't advance depth in this case
                                break;
                            }
                        }
                        Token::ElementEnd {
                            end: ElementEnd::Open,
                            ..
                        } => {
                            depth += 1;
                            break;
                        }
                        Token::Attribute { .. } => (),
                        // there shouldn't have any token but Attribute between ElementStart and ElementEnd
                        token => {
                            return Err(XmlError::UnexpectedToken {
                                token: format!("{:?}", token),
                            });
                        }
                    }
                }
            }
            Token::ElementEnd {
                end: ElementEnd::Close(_, _),
                span,
            } if tag == &span.as_str()[2..span.as_str().len() - 1] => {
                depth -= 1;
                if depth == 0 {
                    return Ok(());
                }
            }
            _ => (),
        }
    }

    Err(XmlError::UnexpectedEof)
}

#[test]
fn test_read_to_end() -> XmlResult<()> {
    let mut reader = Tokenizer::from("<parent><child/></parent>").peekable();

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    assert!(reader.next().is_some()); // "<child"
    read_to_end(&mut reader, "child")?;
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    let mut reader = Tokenizer::from("<parent><child></child></parent>").peekable();

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    assert!(reader.next().is_some()); // "<child"
    read_to_end(&mut reader, "child")?;
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    let mut reader = Tokenizer::from("<parent><child><child/></child></parent>").peekable();

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    assert!(reader.next().is_some()); // "<child"
    read_to_end(&mut reader, "child")?;
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    let mut reader = Tokenizer::from("<parent><child><child></child></child></parent>").peekable();

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    assert!(reader.next().is_some()); // "<child"
    read_to_end(&mut reader, "child")?;
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    Ok(())
}
