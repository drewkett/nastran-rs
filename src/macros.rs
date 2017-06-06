
macro_rules! whole_file {
  ($i:expr, $submac:ident!( $($args:tt)* )) => (
      {
          use errors::ErrorKind;
          use nom::IResult;
            match complete!($i,terminated!($submac!($($args)*), eof!())) {
                IResult::Done(b"",d) => Ok(d),
                IResult::Done(_,_) => Err(ErrorKind::NotPossible("Remaining characters in buffer").into()),
                IResult::Incomplete(_) => Err(ErrorKind::NotPossible("Remaining characters in buffer").into()),
                IResult::Error(e) => Err(e.into()),
            }
      }
  );
  ($i:expr,  $f:expr) => (
    whole_file!($i, call!($f));
  );
}
