use bstr::ByteSlice;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error<'a> {
    #[error("Parse Failure")]
    ParseFailure,
    #[error("Unexpected end to field '{}'",.0.as_bstr())]
    UnexpectedFieldEnd(&'a [u8]),
    #[error("Unexpected char in field '{}'",.0.as_bstr())]
    UnexpectedCharInField(&'a [u8]),
    #[error("Unexpected continuation '{}'",.0.as_bstr())]
    UnexpectedContinuation([u8; 8]),
    #[error("Error on Line {0}: {1}")]
    LineError(usize, Box<Error<'a>>),
    #[error("Unmatched Continuation '{}'",.0.as_bstr())]
    UnmatchedContinuation([u8; 8]),
    #[error("Not Possible '{0}'")]
    NotPossible(&'static str),
    #[error("UTF8 Conversion Error")]
    UTF8ConversionError(#[from] ::std::str::Utf8Error),
    #[error("Error Parsing Integer")]
    ParseIntError(#[from] ::std::num::ParseIntError),
    #[error("Error Parsing Float")]
    ParseFloatError(#[from] ::std::num::ParseFloatError),
}

pub type Result<'a, T> = ::std::result::Result<T, Error<'a>>;