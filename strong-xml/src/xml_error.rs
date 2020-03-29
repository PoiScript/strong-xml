use std::{error::Error, io::Error as IOError, str::Utf8Error, string::FromUtf8Error};
use xmlparser::Error as ParserError;

#[derive(Debug)]
pub enum XmlError {
    IO(IOError),
    Parser(ParserError),
    Utf8(Utf8Error),
    UnexpectedEof,
    UnexpectedToken { token: String },
    TagMismatch { expected: String, found: String },
    MissingField { name: String, field: String },
    UnterminatedEntity { entity: String },
    UnrecognizedSymbol { symbol: String },
    FromStr(Box<dyn Error>),
}

impl From<IOError> for XmlError {
    fn from(err: IOError) -> Self {
        XmlError::IO(err)
    }
}

impl From<Utf8Error> for XmlError {
    fn from(err: Utf8Error) -> Self {
        XmlError::Utf8(err)
    }
}

impl From<FromUtf8Error> for XmlError {
    fn from(err: FromUtf8Error) -> Self {
        XmlError::Utf8(err.utf8_error())
    }
}

impl From<ParserError> for XmlError {
    fn from(err: ParserError) -> Self {
        XmlError::Parser(err)
    }
}

/// Specialized `Result` which the error value is `Error`.
pub type XmlResult<T> = Result<T, XmlError>;
