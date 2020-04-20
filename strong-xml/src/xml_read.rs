use crate::{XmlReader, XmlResult};

pub trait XmlRead<'a>: Sized {
    fn from_reader(reader: &mut XmlReader<'a>) -> XmlResult<Self>;

    fn from_str(text: &'a str) -> XmlResult<Self> {
        let mut reader = XmlReader::new(text);
        Self::from_reader(&mut reader)
    }
}
