use std::collections::{BTreeMap, BTreeSet};
use std::io::Result;
use std::io::Write;

use crate::xml_escape::xml_escape;

pub struct XmlWriter<W: Write> {
    pub inner: W,
    pub used_namespaces: BTreeMap<Option<&'static str>, Vec<&'static str>>,
    pub set_prefixes: Vec<BTreeSet<Option<&'static str>>>,
}

impl<W: Write> XmlWriter<W> {
    pub fn new(inner: W) -> Self {
        XmlWriter {
            inner,
            used_namespaces: BTreeMap::new(),
            set_prefixes: Vec::new(),
        }
    }

    pub fn into_inner(self) -> W {
        self.inner
    }

    pub fn write_element_start(&mut self, tag: &str) -> Result<()> {
        // Add new level to store set prefixes for this scope
        self.set_prefixes.push(BTreeSet::new());
        write!(self.inner, "<{}", tag)
    }

    pub fn write_namespace_declaration(
        &mut self,
        prefix: Option<&'static str>,
        ns: &'static str,
    ) -> Result<()> {
        if !self.is_prefix_defined_as(&prefix, ns) {
            // let writer know that there has been a prefix that has been re/defined.
            self.push_changed_namespace(prefix, ns)?;
            if let Some(prefix) = prefix {
                write!(self.inner, r#" xmlns:{}="{}""#, prefix, xml_escape(ns))
            } else {
                write!(self.inner, r#" xmlns="{}""#, xml_escape(ns))
            }
        } else {
            Ok(())
        }
    }

    pub fn write_attribute(&mut self, key: &str, value: &str) -> Result<()> {
        write!(self.inner, r#" {}="{}""#, key, xml_escape(value))
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

    pub fn write_flatten_text(&mut self, tag: &str, content: &str, is_cdata: bool) -> Result<()> {
        self.write_element_start(tag)?;
        self.write_element_end_open()?;
        if is_cdata {
            self.write_cdata_text(content)?;
        } else {
            self.write_text(content)?;
        }
        self.write_element_end_close(tag)?;
        Ok(())
    }

    pub fn write_element_end_close(&mut self, tag: &str) -> Result<()> {
        // Restore the previous namespace declarations
        self.pop_changed_namespaces()?;
        write!(self.inner, "</{}>", tag)
    }

    pub fn write_element_end_empty(&mut self) -> Result<()> {
        // Restore the previous namespace declarations
        self.pop_changed_namespaces()?;
        write!(self.inner, "/>")
    }

    pub fn is_prefix_defined_as(&mut self, prefix: &Option<&str>, namespace: &str) -> bool {
        match self.used_namespaces.get(prefix) {
            Some(scope) => scope.last() == Some(&namespace),
            _ => false,
        }
    }

    pub fn get_namespace(&mut self, prefix: &Option<&str>) -> Option<&'static str> {
        match self.used_namespaces.get(prefix) {
            Some(scope) => {
                if let Some(&namespace) = scope.last() {
                    Some(namespace)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn pop_changed_namespaces(&mut self) -> Result<()> {
        use std::io::{Error, ErrorKind};

        // Go and get the current set_prefixes for this scope
        // and pop of the last namespace for each prefix
        if let Some(set_prefixes) = self.set_prefixes.pop() {
            set_prefixes
                .iter()
                .map(|pfx| -> Result<()> {
                    match self.used_namespaces.get_mut(pfx) {
                        Some(v) => {
                            if let Some(_) = v.pop() {
                                Ok(())
                            } else {
                                Err(Error::new(
                                    ErrorKind::Other,
                                    "Prefix state could not be popped",
                                ))
                            }
                        }
                        None => Err(Error::new(
                            ErrorKind::Other,
                            "Prefix does not exist in scope",
                        )),
                    }
                })
                .collect::<Result<Vec<()>>>()?;
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::Other,
                "Failed to restore previous prefix scope",
            ))
        }
    }

    fn push_changed_namespace(
        &mut self,
        prefix: Option<&'static str>,
        namespace: &'static str,
    ) -> Result<()> {
        use std::io::{Error, ErrorKind};

        // Get the current prefix scope off of the stack
        let set_prefixes = if let Some(prefixes) = self.set_prefixes.last_mut() {
            prefixes
        } else {
            return Err(Error::new(
                ErrorKind::Other,
                "Failed to get current prefix scope",
            ));
        };

        // Push prefix onto stack for namespace 
        // or create new stack with namespace as only element
        if let Some(v) = self.used_namespaces.get_mut(&prefix) {
            v.push(namespace);
        } else {
            self.used_namespaces.insert(prefix, vec![namespace]);
        }
        set_prefixes.insert(prefix);

        Ok(())
    }
}
