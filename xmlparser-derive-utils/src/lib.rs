pub mod read_text;
pub mod read_till_element_start;
pub mod read_to_end;
pub mod xml_error;
pub mod xml_escape;
pub mod xml_read;
pub mod xml_unescape;
pub mod xml_write;

pub use self::read_text::read_text;
pub use self::read_till_element_start::read_till_element_start;
pub use self::read_to_end::read_to_end;
pub use self::xml_error::{XmlError, XmlResult};
pub use self::xml_escape::xml_escape;
pub use self::xml_read::XmlRead;
pub use self::xml_unescape::xml_unescape;
pub use self::xml_write::XmlWrite;

use std::iter::Peekable;
use xmlparser::Tokenizer;

pub type XmlReader<'a> = Peekable<Tokenizer<'a>>;
