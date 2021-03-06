use std::io;

use crate::bdf::parser::Field;

use bstr::ByteSlice;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Embedded Space in field")]
    EmbeddedSpace,
    #[error("Unexpected character {}",(&[*.0][..]).as_bstr())]
    UnexpectedChar(u8),
    #[error("Text field greater than 8 chars '{}'",.0.as_bstr())]
    TextTooLong(Vec<u8>),
    #[error("Field is not valid")]
    InvalidField,
    #[error("Whole line not parsed. Remaining '{}'",.0.as_bstr())]
    UnparsedChars(Vec<u8>),
    #[error("Unmatched continuation")]
    UnmatchedContinuation([u8; 7]),
    #[error("Unexpected Card Type. Expected '{}' Found '{}'",.0.as_bstr(),.1.as_bstr())]
    UnexpectedCardType([u8; 7], [u8; 7]),
    #[error("Unexpected Card Type. Expected '{0}' Found '{1:?}'")]
    UnexpectedField(&'static str, Field),
    #[error("Unexpected field for dof. Found '{0:?}'")]
    UnexpectedDOF(Field),
    #[error("Invalid specification of e ({0}),g ({1}) and nu ({2}) for material card ")]
    InvalidMaterialCard(Field, Field, Field),
    #[error("Duplicate Card")]
    DuplicateCard,
    #[error("Error reading datfile : {0}")]
    IO(#[from] io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
