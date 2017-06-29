
use nom;

error_chain!{
    errors {
        ParseFailure
        UnexpectedFieldEnd
        UnexpectedCharInField
        UnexpectedContinuation
        UnmatchedContinuation(t: String)
        NotPossible(t: &'static str)
    }
    foreign_links {
        UTF8ConversionError(::std::str::Utf8Error);
        ParseIntError(::std::num::ParseIntError);
        ParseFloatError(::std::num::ParseFloatError);
    }
}

impl From<nom::ErrorKind> for Error {
    fn from(_: nom::ErrorKind) -> Error {
        ErrorKind::ParseFailure.into()
    }
}
