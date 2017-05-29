
use nom;

error_chain!{
    errors {
        ParseFailure {}
        NotPossible(t: &'static str)
    }
}

impl From<nom::ErrorKind> for Error {
    fn from(_: nom::ErrorKind) -> Error {
        ErrorKind::ParseFailure.into()
    }
}