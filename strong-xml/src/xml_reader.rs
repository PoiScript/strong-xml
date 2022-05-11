use std::borrow::Cow;
use std::fmt::format;
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

/// Xml Nma
///
/// Holds both the namespace prefix and the local name
/// of a node or attribute
#[derive(PartialEq, Eq)]
pub struct XmlName<'a> {
    pub prefix: &'a str,
    pub local: &'a str,
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
    fn make_name<'r>(prefix: &'r str, local: &'r str) -> Cow<'r, str> {
        if prefix.is_empty() {
            Cow::Borrowed(local)
        } else {
            Cow::Owned(prefix.to_owned() + ":" + local)
        }
    }

    #[inline]
    pub fn read_text(
        &mut self,
        end_prefix: &'a str,
        end_local: &'a str,
    ) -> XmlResult<Cow<'a, str>> {
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
                    end: ElementEnd::Close(prefix, local),
                    ..
                } => {
                    if prefix.as_str() == end_prefix && local.as_str() == end_local {
                        break;
                    } else {
                        return Err(XmlError::TagMismatch {
                            expected: Self::make_name(end_prefix, end_local).to_string(),
                            found: Self::make_name(prefix.as_str(), local.as_str()).to_string(),
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
    pub fn read_till_element_start(&mut self, end_prefix: &str, end_local: &str) -> XmlResult<()> {
        while let Some(token) = self.next() {
            match token? {
                Token::ElementStart { prefix, local, .. } => {
                    if prefix.as_str() == end_prefix && local.as_str() == end_local {
                        break;
                    } else {
                        self.read_to_end(prefix.as_str(), local.as_str())?;
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
    pub fn find_attribute(&mut self) -> XmlResult<Option<(&'a str, &'a str, Cow<'a, str>)>> {
        if let Some(token) = self.tokenizer.peek() {
            match token {
                Ok(Token::Attribute {
                    prefix,
                    local,
                    value,
                    ..
                }) => {
                    let prefix = prefix.as_str();
                    let local = local.as_str();
                    let value = Cow::Borrowed(value.as_str());
                    self.next();
                    return Ok(Some((prefix, local, value)));
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
    pub fn find_element_start(
        &mut self,
        end_tag: Option<(&str, &str)>,
    ) -> XmlResult<Option<(&'a str, &'a str)>> {
        while let Some(token) = self.tokenizer.peek() {
            match token {
                Ok(Token::ElementStart { prefix, local, .. }) => {
                    return Ok(Some((prefix.as_str(), local.as_str())));
                }
                Ok(Token::ElementEnd {
                    end: ElementEnd::Close(prefix, local),
                    ..
                }) if end_tag.is_some() => {
                    let (end_prefix, end_local) = end_tag.unwrap();
                    if prefix.as_str() == end_prefix && local.as_str() == end_local {
                        self.next();
                        return Ok(None);
                    } else {
                        return Err(XmlError::TagMismatch {
                            expected: Self::make_name(end_prefix, end_local).to_string(),
                            found: Self::make_name(prefix.as_str(), local.as_str()).to_string(),
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
    pub fn read_to_end(&mut self, end_prefix: &str, end_local: &str) -> XmlResult<()> {
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
                Token::ElementStart { prefix, local, .. }
                    if prefix.as_str() == end_prefix && local.as_str() == end_local =>
                {
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
                    end: ElementEnd::Close(prefix, local),
                    ..
                } if prefix.as_str() == end_prefix && local.as_str() == end_local => {
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

impl<'a> XmlName<'a> {
    #[inline]
    pub fn new(prefix: &'a str, local: &'a str) -> XmlName<'a> {
        XmlName { prefix, local }
    }

    #[inline]
    pub fn cmp_raw<P>(&self, prefix: P, local: &'a str) -> bool
    where
        P: Into<Option<&'a str>>,
    {
        // this make the method able to be called like
        // cmp_raw(None, "local") or cmp_raw("prefix", "local")
        let prefix: Option<_> = prefix.into();
        (prefix == None && self.prefix == "") || prefix == Some(self.prefix) && local == self.local
    }
}

#[test]
fn read_text() -> XmlResult<()> {
    let mut reader = XmlReader::new("<parent></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert_eq!(reader.read_text("", "parent")?, "");
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent>text</parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert_eq!(reader.read_text("", "parent")?, "text");
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent attr=\"value\">text</parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert_eq!(reader.read_text("", "parent")?, "text");
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent attr=\"value\">&quot;&apos;&lt;&gt;&amp;</parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert_eq!(reader.read_text("", "parent")?, r#""'<>&"#);
    assert!(reader.next().is_none());

    let mut reader = XmlReader::new("<parent><![CDATA[]]></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert_eq!(reader.read_text("", "parent")?, "");
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent><![CDATA[text]]></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert_eq!(reader.read_text("", "parent")?, "text");
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent attr=\"value\"><![CDATA[text]]></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert_eq!(reader.read_text("", "parent")?, "text");
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent attr=\"value\"><![CDATA[<foo></foo>]]></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert_eq!(reader.read_text("", "parent")?, "<foo></foo>");
    assert!(reader.next().is_none());

    reader =
        XmlReader::new("<parent attr=\"value\"><![CDATA[&quot;&apos;&lt;&gt;&amp;]]></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert_eq!(reader.read_text("", "parent")?, "&quot;&apos;&lt;&gt;&amp;");
    assert!(reader.next().is_none());

    Ok(())
}

#[test]
fn read_till_element_start() -> XmlResult<()> {
    let mut reader = XmlReader::new("<tag/>");

    reader.read_till_element_start("", "tag")?;
    assert!(reader.next().is_some()); // "/>"
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent><skip/><tag/></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    reader.read_till_element_start("", "tag")?;
    assert!(reader.next().is_some()); // "/>"
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent><skip></skip><tag/></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    reader.read_till_element_start("", "tag")?;
    assert!(reader.next().is_some()); // "/>"
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent><skip><skip/></skip><tag/></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    reader.read_till_element_start("", "tag")?;
    assert!(reader.next().is_some()); // "/>"
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent><skip><skip></skip></skip><tag/></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    reader.read_till_element_start("", "tag")?;
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
    reader.read_to_end("", "child")?;
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent><child></child></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    assert!(reader.next().is_some()); // "<child"
    reader.read_to_end("", "child")?;
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent><child><child/></child></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    assert!(reader.next().is_some()); // "<child"
    reader.read_to_end("", "child")?;
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    reader = XmlReader::new("<parent><child><child></child></child></parent>");

    assert!(reader.next().is_some()); // "<parent"
    assert!(reader.next().is_some()); // ">"
    assert!(reader.next().is_some()); // "<child"
    reader.read_to_end("", "child")?;
    assert!(reader.next().is_some()); // "</parent>"
    assert!(reader.next().is_none());

    Ok(())
}
