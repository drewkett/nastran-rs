

macro_rules! take_m_n_while (
    ($i:expr, $m:expr, $n: expr, $submac:ident!( $($args:tt)* )) => (
        {
            use std::cmp::min;
            use nom::{InputIter, Slice};
            let input = $i;
            let mn: usize = $m;
            let mx: usize = $n;
            let l = min(input.len(),mx);
            if l < mn {
                return IResult::Incomplete(nom::Needed::Size(mn-l))
            }
            let temp = input.slice(..l);
            match temp.position(|c| !$submac!(c, $($args)*)) {
                Some(j) if j + 1 < mn =>  IResult::Incomplete(nom::Needed::Size(mn-j-1)),
                Some(j) => IResult::Done(input.slice(j..), input.slice(..j)),
                None    => IResult::Done(input.slice(l..), input.slice(..l))
            }
        }
    );
    ($input:expr, $m:expr, $n: expr, $f:expr) => (
        take_m_n_while!($input, $m, $n, call!($f));
    );
);

macro_rules! char_if (
  ($i:expr, $submac:ident!( $($args:tt)* )) => (
      {
          let input = $i;
          if $i.len() == 0 {
              return IResult::Incomplete(nom::Needed::Size(1))
          }
            match ($i).iter_elements().next().map(|&c| $submac!(c, $($args)*)) {
                None        => IResult::Incomplete::<_, _>(nom::Needed::Size(1)),
                Some(false) => IResult::Error(error_position!(nom::ErrorKind::Char, $i)),
                //the unwrap should be safe here
                Some(true)  => IResult::Done($i.slice(1..), $i.iter_elements().next().unwrap())
            }
      }
  );
  ($input:expr,  $f:expr) => (
    char_if!($input, call!($f));
  );
);

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
