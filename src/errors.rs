
use nom;

error_chain!{
    errors { ParseFailure }
}

impl From<nom::ErrorKind> for Error {
    fn from(_: nom::ErrorKind) -> Error {
        ErrorKind::ParseFailure.into()
    }
}