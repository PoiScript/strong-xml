use std::io::Write;

use crate::XmlResult;

pub trait XmlWrite: Sized {
    fn to_string(&self) -> XmlResult<String>;
    fn to_writer<W: Write>(&self, writer: W) -> XmlResult<()>;
}
