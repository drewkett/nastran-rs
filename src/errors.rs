
use nom;

quick_error!{
    #[derive(Debug)]
    pub enum Error {
        ParseFailure
        UnexpectedFieldEnd(field: String)
        UnexpectedCharInField(field: String)
        UnexpectedContinuation(continuation: String)
        LineError(line: usize, e: Box<Error>)
        UnmatchedContinuation(continuation: String)
        NotPossible(msg: &'static str)
        UTF8ConversionError(e: ::std::str::Utf8Error) {
            from()
        }
        OP2ParseError(e: nom::ErrorKind) {
            from()
        }
        ParseIntError(e: ::std::num::ParseIntError) {
            from()
        }
        ParseFloatError(e: ::std::num::ParseFloatError) {
            from()
        }
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;
