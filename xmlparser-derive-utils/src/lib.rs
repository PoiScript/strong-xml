pub mod read_text;
pub mod read_to_end;
pub mod xml_error;

pub use self::read_text::read_text;
pub use self::read_to_end::read_to_end;
pub use self::xml_error::{XmlError, XmlResult};
