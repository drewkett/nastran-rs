use std::fmt;

use nom;

#[derive(Debug)]
pub enum Error {
    ParseFailure,
    UnexpectedFieldEnd(String),
    UnexpectedCharInField(String),
    UnexpectedContinuation(String),
    LineError(usize, Box<Error>),
    UnmatchedContinuation(String),
    NotPossible(&'static str),
    UTF8ConversionError(::std::str::Utf8Error),
    OP2ParseError(nom::ErrorKind),
    ParseIntError(::std::num::ParseIntError),
    ParseFloatError(::std::num::ParseFloatError),
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Self {
        Error::UTF8ConversionError(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Error::ParseIntError(e)
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(e: std::num::ParseFloatError) -> Self {
        Error::ParseFloatError(e)
    }
}
impl From<nom::ErrorKind> for Error {
    fn from(e: nom::ErrorKind) -> Self {
        Error::OP2ParseError(e)
    }
}
