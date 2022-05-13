use std::borrow::Cow;
use std::iter::{Iterator, Peekable};

use xmlparser::ElementEnd;
use xmlparser::Error;
use xmlparser::Token;
use xmlparser::Tokenizer;

use crate::xml_unescape::xml_unescape;
use crate::{XmlError, XmlResult};

/// Xml Reader
///
/// It behaves almost exactly like `xmlparser::Tokenizer::from("...").peekable()`
/// but with some helper functions.
pub struct XmlReader<'a> {
    tokenizer: Peekable<Tokenizer<'a>>,
}

impl<'a> XmlReader<'a> {
    #[inline]
    pub fn new(text: &'a str) -> XmlReader<'a> {
        XmlReader {
            tokenizer: Tokenizer::from(text).peekable(),
        }
    }

    #[inline]
    pub fn next(&mut self) -> Option<Result<Token<'a>, Error>> {
        self.tokenizer.next()
    }

    #[inline]
    pub fn peek(&mut self) -> Option<&Result<Token<'a>, Error>> {
        self.tokenizer.peek()
    }

    #[inline]
    pub fn read_text(&mut self, end_tag: &'a str) -> XmlResult<Cow<'a, str>> {
        let mut res = None;

        while let Some(token) = self.next() {
            match token? {
                Token::ElementEnd {
                    end: ElementEnd::Open,
                    ..
                }
                | Token::Attribute { .. } => (),
                Token::Text { text } => {
                    res = Some(xml_unescape(text.as_str())?);
                }
                Token::Cdata { text, .. } => {
                    res = Some(Cow::Borrowed(text.as_str()));
                }
                Token::ElementEnd {
                    end: ElementEnd::Close(_, _),
                    span,
                } => {
                    if end_tag == &span[2..span.len() - 1] {
                        break;
                    } else {
                        return Err(XmlError::TagMismatch {
                            expected: end_tag.to_owned(),
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

    #[inline]
    pub fn read_till_element_start(&mut self, end_tag: &str) -> XmlResult<()> {
        while let Some(token) = self.next() {
            match token? {
                Token::ElementStart { span, .. } => {
                    let tag = &span.as_str()[1..];
                    if end_tag == tag {
                        break;
                    } else {
                        self.read_to_end(tag)?;
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

    #[inline]
    pub fn find_attribute(&mut self) -> XmlResult<Option<(&'a str, Cow<'a, str>)>> {
        if let Some(token) = self.tokenizer.peek() {
            match token {
                Ok(Token::Attribute {
                    prefix,
                    local,
                    value,
                    span,
                }) => {
                    let length = local.len()
                        + if !prefix.is_empty() {
                            prefix.len() + 1
                        } else {
                            0
                        };
                    let name = &span.as_str()[0..length];
                    let value = Cow::Borrowed(value.as_str());
                    self.next();
                    return Ok(Some((name, value)));
                }
                Ok(Token::ElementEnd {
                    end: ElementEnd::Open,
                    ..
                })
                | Ok(Token::ElementEnd {
                    end: ElementEnd::Empty,
                    ..
                }) => return Ok(None),
                Ok(token) => {
                    return Err(XmlError::UnexpectedToken {
                        token: format!("{:?}", token),
                    })
                }
                Err(_) => {
                    // we have call .peek() above, and it's safe to use unwrap
                    self.next().unwrap()?;
                }
            }
        }

        Err(XmlError::UnexpectedEof)
    }

    #[inline]
    pub fn find_element_start(&mut self, end_tag: Option<&str>) -> XmlResult<Option<&'a str>> {
        while let Some(token) = self.tokenizer.peek() {
            match token {
                Ok(Token::ElementStart { span, .. }) => {
                    let name = &span.as_str()[1..];
                    return Ok(Some(name));
                }
                Ok(Token::ElementEnd {
                    end: ElementEnd::Close(_, _),
                    span,
                }) if end_tag.is_some() => {
                    let end_tag = end_tag.unwrap();

                    let name = &span.as_str()[2..span.len() - 1];
                    if end_tag == name {
                        self.next();
                        return Ok(None);
                    } else {
                        return Err(XmlError::TagMismatch {
                            expected: end_tag.to_string(),
                            found: name.to_string(),
                        });
                    }
                }
                Ok(Token::ElementEnd { .. }) | Ok(Token::Attribute { .. }) => {
                    return Err(XmlError::UnexpectedToken {
                        token: format!("{:?}", token),
                    })
                }
                _ => {
                    // we have call .peek() above, and it's safe to use unwrap
                    self.next().unwrap()?;
                }
            }
        }
        Err(XmlError::UnexpectedEof)
    }

    #[inline]
    pub fn read_to_end(&mut self, end_tag: &str) -> XmlResult<()> {
        while let Some(token) = self.next() {
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

        while let Some(token) = self.next() {
            match token? {
                Token::ElementStart { span, .. } if end_tag == &span.as_str()[1..] => {
                    while let Some(token) = self.next() {
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
                } if end_tag == &span[2..span.len() - 1] => {
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
}

#[test]
fn read_text() -> XmlResult<()> {
    let mut reader = XmlReader::new("<parent></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert_eq!(reader.read_text("parent")?, "");
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent>text</parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert_eq!(reader.read_text("parent")?, "text");
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent attr=\"value\">text</parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert_eq!(reader.read_text("parent")?, "text");
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent attr=\"value\">&quot;&apos;&lt;&gt;&amp;</parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert_eq!(reader.read_text("parent")?, r#""'<>&"#);
    assert!(reader.next().is_none());

    let mut reader = XmlReader::new("<parent><![CDATA[]]></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert_eq!(reader.read_text("parent")?, "");
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent><![CDATA[text]]></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert_eq!(reader.read_text("parent")?, "text");
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent attr=\"value\"><![CDATA[text]]></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert_eq!(reader.read_text("parent")?, "text");
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent attr=\"value\"><![CDATA[<foo></foo>]]></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert_eq!(reader.read_text("parent")?, "<foo></foo>");
    assert!(reader.next().is_none());

    reader =
        XmlReader::new("<parent attr=\"value\"><![CDATA[&quot;&apos;&lt;&gt;&amp;]]></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert_eq!(reader.read_text("parent")?, "&quot;&apos;&lt;&gt;&amp;");
    assert!(reader.next().is_none());

    Ok(())
}

#[test]
fn read_till_element_start() -> XmlResult<()> {
    let mut reader = XmlReader::new("<tag/>");

    reader.read_till_element_start("tag")?;
    assert!(reader.next().is_some()); // "/>"
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent><skip/><tag/></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    reader.read_till_element_start("tag")?;
    assert!(reader.next().is_some()); // "/>"
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent><skip></skip><tag/></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    reader.read_till_element_start("tag")?;
    assert!(reader.next().is_some()); // "/>"
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent><skip><skip/></skip><tag/></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    reader.read_till_element_start("tag")?;
    assert!(reader.next().is_some()); // "/>"
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent><skip><skip></skip></skip><tag/></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    reader.read_till_element_start("tag")?;
    assert!(reader.next().is_some()); // "/>"
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    Ok(())
}

#[test]
fn read_to_end() -> XmlResult<()> {
    let mut reader = XmlReader::new("<parent><child/></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    assert!(reader.next().is_some()); // "<child"
    reader.read_to_end("child")?;
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent><child></child></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    assert!(reader.next().is_some()); // "<child"
    reader.read_to_end("child")?;
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent><child><child/></child></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    assert!(reader.next().is_some()); // "<child"
    reader.read_to_end("child")?;
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent><child><child></child></child></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    assert!(reader.next().is_some()); // "<child"
    reader.read_to_end("child")?;
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    Ok(())
}
