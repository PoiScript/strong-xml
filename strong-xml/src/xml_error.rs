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
    FromStr(Box<dyn Error + Send + Sync>),
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

impl Error for XmlError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use XmlError::*;
        match self {
            IO(e) => Some(e),
            Parser(e) => Some(e),
            Utf8(e) => Some(e),
            FromStr(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

impl std::fmt::Display for XmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use XmlError::*;
        match self {
            IO(e) => write!(f, "I/O error: {}", e),
            Parser(e) => write!(f, "XML parser error: {}", e),
            Utf8(e) => write!(f, "invalid UTF-8: {}", e),
            UnexpectedEof => f.write_str("unexpected end of file"),
            UnexpectedToken { token } => write!(f, "unexpected token in XML: {:?}", token),
            TagMismatch { expected, found } => write!(
                f,
                "mismatched XML tag; expected {:?}, found {:?}",
                expected, found
            ),
            MissingField { name, field } => {
                write!(f, "missing field in XML of {:?}: {:?}", name, field)
            }
            UnterminatedEntity { entity } => write!(f, "unrecognized XML entity: {}", entity),
            UnrecognizedSymbol { symbol } => write!(f, "unrecognized XML symbol: {}", symbol),
            FromStr(e) => write!(f, "error parsing XML value: {}", e),
        }
    }
}
