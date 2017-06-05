
use nom;
use std::num;

error_chain!{
    errors {
        ParseFailure {}
        NotPossible(t: &'static str)
    }
    foreign_links {
        ParseIntError(num::ParseIntError);
        ParseFloatError(num::ParseFloatError);
    }
}

impl From<nom::ErrorKind> for Error {
    fn from(_: nom::ErrorKind) -> Error {
        ErrorKind::ParseFailure.into()
    }
}
