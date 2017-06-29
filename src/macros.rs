

macro_rules! whole_file {
    ($i:expr, $submac:ident!( $($args:tt)* )) => (
        {
            use nom::IResult;
            use errors::Error;
            match complete!($i,terminated!($submac!($($args)*), eof!())) {
                IResult::Done(b"",d) => Ok(d),
                IResult::Done(_,_) => Err(Error::NotPossible("Remaining characters in buffer")),
                IResult::Incomplete(_) => Err(Error::NotPossible("Remaining characters in buffer")),
                IResult::Error(e) => Err(e.into()),
            }
        }
    );
    ($i:expr,  $f:expr) => (
        whole_file!($i, call!($f));
    );
}
