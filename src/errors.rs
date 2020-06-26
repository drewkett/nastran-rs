use std::fmt;

use nom;

#[derive(Debug)]
pub enum Error<'a> {
    ParseFailure,
    UnexpectedFieldEnd(&'a [u8]),
    UnexpectedCharInField(&'a [u8]),
    UnexpectedContinuation(&'a [u8]),
    LineError(usize, Box<Error<'a>>),
    UnmatchedContinuation(&'a [u8]),
    NotPossible(&'static str),
    UTF8ConversionError(::std::str::Utf8Error),
    OP2ParseError(nom::ErrorKind),
    ParseIntError(::std::num::ParseIntError),
    ParseFloatError(::std::num::ParseFloatError),
}

pub type Result<'a, T> = ::std::result::Result<T, Error<'a>>;

impl<'a> fmt::Display for Error<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}

impl<'a> From<std::str::Utf8Error> for Error<'a> {
    fn from(e: std::str::Utf8Error) -> Self {
        Error::UTF8ConversionError(e)
    }
}

impl<'a> From<std::num::ParseIntError> for Error<'a> {
    fn from(e: std::num::ParseIntError) -> Self {
        Error::ParseIntError(e)
    }
}

impl<'a> From<std::num::ParseFloatError> for Error<'a> {
    fn from(e: std::num::ParseFloatError) -> Self {
        Error::ParseFloatError(e)
    }
}

impl<'a> From<nom::ErrorKind> for Error<'a> {
    fn from(e: nom::ErrorKind) -> Self {
        Error::OP2ParseError(e)
    }
}
