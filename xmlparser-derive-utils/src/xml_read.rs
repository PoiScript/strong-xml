use crate::{XmlReader, XmlResult};

pub trait XmlRead: Sized {
    fn from_str(string: &str) -> XmlResult<Self>;
    fn from_reader(reader: &mut XmlReader) -> XmlResult<Self>;
}
