use std::{
    io::Error as IOError,
    num::ParseIntError,
    str::{ParseBoolError, Utf8Error},
    string::FromUtf8Error,
};

use xmlparser::Error as ParserError;

#[derive(Debug)]
pub enum XmlError {
    IO(IOError),
    Parser(ParserError),
    ParseInt(ParseIntError),
    ParseBool(ParseBoolError),
    Utf8(Utf8Error),
    UnexpectedEof,
    UnexpectedToken { token: String },
    TagMismatch { expected: String, found: String },
    MissingField { name: String, field: String },
    UnknownValue { expected: String, found: String },
    UnterminatedEntity { entity: String },
    UnrecognizedSymbol { symbol: String },
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

impl From<ParseIntError> for XmlError {
    fn from(err: ParseIntError) -> Self {
        XmlError::ParseInt(err)
    }
}

impl From<ParseBoolError> for XmlError {
    fn from(err: ParseBoolError) -> Self {
        XmlError::ParseBool(err)
    }
}

impl From<ParserError> for XmlError {
    fn from(err: ParserError) -> Self {
        XmlError::Parser(err)
    }
}

/// Specialized `Result` which the error value is `Error`.
pub type XmlResult<T> = Result<T, XmlError>;
