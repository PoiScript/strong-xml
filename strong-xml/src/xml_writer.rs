use std::io::Result;
use std::io::Write;

use crate::xml_escape::xml_escape;

pub struct XmlWriter<W: Write> {
    pub inner: W,
}

impl<W: Write> XmlWriter<W> {
    pub fn new(inner: W) -> Self {
        XmlWriter { inner }
    }

    pub fn into_inner(self) -> W {
        self.inner
    }

    pub fn write_element_start(&mut self, prefix: Option<&str>, tag: &str) -> Result<()> {
        if let Some(prefix) = prefix {
            write!(self.inner, "<{}:{}", prefix, tag)
        } else {
            write!(self.inner, "<{}", tag)
        }
    }

    pub fn write_attribute(&mut self, prefix: Option<&str>, key: &str, value: &str) -> Result<()> {
        if let Some(prefix) = prefix {
            write!(self.inner, r#" {}:{}="{}""#, prefix, key, xml_escape(value))
        } else {
            write!(self.inner, r#" {}="{}""#, key, xml_escape(value))
        }
    }

    pub fn write_text(&mut self, content: &str) -> Result<()> {
        write!(self.inner, "{}", xml_escape(content))
    }

    pub fn write_cdata_text(&mut self, content: &str) -> Result<()> {
        write!(self.inner, "<![CDATA[{}]]>", content)
    }

    pub fn write_element_end_open(&mut self) -> Result<()> {
        write!(self.inner, ">")
    }

    pub fn write_flatten_text(&mut self, prefix: Option<&str>, tag: &str, content: &str, is_cdata: bool) -> Result<()> {
        self.write_element_start(prefix, tag)?;
        self.write_element_end_open()?;
        if is_cdata {
            self.write_cdata_text(content)?;
        } else {
            self.write_text(content)?;
        }
        self.write_element_end_close(prefix, tag)?;
        Ok(())
    }

    pub fn write_element_end_close(&mut self, prefix: Option<&str>, tag: &str) -> Result<()> {
        if let Some(prefix) = prefix {
            write!(self.inner, "</{}:{}>", prefix, tag)
        } else {
            write!(self.inner, "</{}>", tag)
        }
    }

    pub fn write_element_end_empty(&mut self) -> Result<()> {
        write!(self.inner, "/>")
    }
}
