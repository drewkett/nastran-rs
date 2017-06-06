
use std::str;

use super::{Field, Card};

use nom::{self, IResult, Slice, InputIter, is_space, is_alphanumeric, is_alphabetic, is_digit, digit};

fn parse_nastran_float(value: &[u8], exponent: &[u8]) -> f32 {
    let j = value.len();
    let n = j + exponent.len() + 1;
    let mut temp = [b' '; 80];
    temp[..j].copy_from_slice(value);
    temp[j] = b'e';
    temp[j + 1..n].copy_from_slice(exponent);
    let s = unsafe { str::from_utf8_unchecked(&temp[..n]) };
    s.parse::<f32>().expect("Failed to parse nastran float")
}

named!(field_string<Field>,map!(
    recognize!(tuple!(char_if!(is_alphabetic),take_m_n_while!(0,7,is_alphanumeric))),
    Field::String));
named!(field_string_double<Field>,map!(
    terminated!(
        recognize!( tuple!(char_if!(is_alphabetic),take_m_n_while!(0,7,is_alphanumeric)))
        ,tuple!(take_while!(is_space),tag!("*"))
    ), Field::String));
named!(field_cont_single<Field>,map!(
    preceded!(tag!("+"),take_while!(move |c| c == b' ' || is_alphanumeric(c))),
    Field::Continuation));
named!(field_cont_double<Field>,map!(
    preceded!(tag!("*"),take_while!(move |c| c == b' ' || is_alphanumeric(c))),
Field::Continuation));
named!(field_cont_anon<Field>,map!(
    take_m_n_while!(0,8,move |c| is_alphanumeric(c) || c == b' '),
    Field::Continuation));
named!(field_cont<Field>,alt_complete!(field_cont_single|field_cont_double|field_cont_anon));
named!(field_float<Field>,map!(my_float, Field::Float));
named!(field_double<Field>,map!(my_double, Field::Double));

named!(decimal_float_value, recognize!(alt!(
          delimited!(digit, tag!("."), opt!(complete!(digit)))
          | terminated!(tag!("."), digit)
        )
));

named!(float_exponent, recognize!(tuple!(
          one_of!("eE"),
          opt!(one_of!("+-")),
          digit
          )
));

named!(double_exponent, recognize!(tuple!(
          one_of!("dD"),
          opt!(one_of!("+-")),
          digit
          )
));

fn my_float(input: &[u8]) -> IResult<&[u8], f32> {
    flat_map!(input,
    recognize!(
      tuple!(
        opt!(one_of!("+-")),
        alt!(
            terminated!(decimal_float_value,opt!(complete!(float_exponent))) |
            terminated!(digit,float_exponent)
        )
      )
    ),
    parse_to!(f32)
  )
}

fn my_double(input: &[u8]) -> IResult<&[u8], f64> {
    flat_map!(input,
    recognize!(
      tuple!(
        opt!(one_of!("+-")),
        alt!(
            terminated!(decimal_float_value,double_exponent) |
            terminated!(digit,double_exponent)
        )
      )
    ),
    parse_to!(f64)
  )
}

named!(field_nastran_float<Field>,map!(
    tuple!(
        recognize!(
            tuple!(
            opt!(one_of!("+-")),
                alt!(
                    delimited!(digit, tag!("."), opt!(digit))
                    | terminated!(tag!("."), digit)
                )
            )
        ),
        recognize!(tuple!(one_of!("+-"),digit))
    ),
    |(value,exponent)| Field::Float(parse_nastran_float(value,exponent))
));

named!(field_integer<Field>,map!(
    flat_map!(
        recognize!(
            alt!(
                preceded!(tag!("-"),take_m_n_while!(1,7,is_digit))|
                take_m_n_while!(1,8,is_digit)
            )
        ),
        parse_to!(i32)
    )
    ,|i| Field::Int(i))
);

macro_rules! pad_space_eof(
    ($i:expr, $submac:ident!( $($args:tt)* )) => (
        delimited!($i,
            take_while!(is_space),
            $submac!($($args)*),
            tuple!(take_while!(is_space),eof!())
        )
    );
    ($i:expr, $f:expr) => (
        pad_space_eof!($i, call!($f));
    );
);

named!(pub short_field<Field>,
    alt_complete!(
        pad_space_eof!(field_float) |
        pad_space_eof!(field_nastran_float) |
        pad_space_eof!(field_integer) |
        pad_space_eof!(field_string) |
        value!(Field::Blank,tuple!(take_while!(is_space),eof!())
    )
));

named!(pub short_field_cont<Field>,
    alt_complete!(
        pad_space_eof!(field_float) |
        pad_space_eof!(field_nastran_float) |
        pad_space_eof!(field_integer) |
        pad_space_eof!(field_string) |
        terminated!(field_cont_single,eof!()) |
        terminated!(field_cont_double,eof!()) |
        value!(Field::Blank,tuple!(take_while!(is_space),eof!()))
    )
);

named!(long_field<Field>,
    alt_complete!(
        pad_space_eof!(field_float) |
        pad_space_eof!(field_double) |
        pad_space_eof!(field_nastran_float) |
        pad_space_eof!(field_integer) |
        pad_space_eof!(field_string) |
        value!(Field::Blank,tuple!(take_while!(is_space),eof!()))
    )
);

named!(pub first_field<Card>, alt_complete!(
    map!(pad_space_eof!(field_string),|field| Card::from_first_field(field,false)) |
    map!(pad_space_eof!(field_string_double),|field| Card::from_first_field(field,true)) |
    map!(terminated!(field_cont_single,eof!()),|field| Card::from_first_field(field,false)) |
    map!(terminated!(field_cont_double,eof!()),|field| Card::from_first_field(field,true)) |
    value!(Card::from_first_field(Field::Blank,false),tuple!(take_while!(is_space),eof!()))
));

named!(pub field_8<Field>, flat_map!(take_m_n_while!(0,8,move |c| c!= b'\t'),short_field));
named!(pub field_8_cont<Field>, flat_map!(take_m_n_while!(0,8,move |c| c!= b'\t'),field_cont));
named!(pub field_16<Field>, flat_map!(take_m_n_while!(0,16,move |c| c!= b'\t'),long_field));

#[cfg(test)]
mod tests {
    extern crate test;
    use test::Bencher;
    use std::fmt::Debug;
    use nom::{ErrorKind, Err};

    use super::*;

    #[bench]
    fn bench_field_nastran_float(b: &mut Bencher) {
        b.iter(|| field_nastran_float(b"11.22+7"));
    }

    #[bench]
    fn bench_field_float(b: &mut Bencher) {
        b.iter(|| field_float(b"11.22e+7"));
    }
    fn assert_done<T: Debug + PartialEq>(result: T, test: IResult<&[u8], T>) {
        assert_eq!(IResult::Done(&b""[..],result),test);
    }

    fn assert_error<T: Debug + PartialEq>(result: Err<u32>, test: IResult<&[u8], T>) {
        assert_eq!(IResult::Error(result),test);
    }

    #[test]
    fn test_parse() {
        assert_done(Field::Float(1.23), short_field(b" 1.23 "));
        assert_done(Field::Float(1.24), short_field(b" 1.24"));
        assert_done(Field::Float(1.25), short_field(b"1.25"));
        assert_error(error_code!(ErrorKind::Alt), short_field(b"1252341551"));
        assert_done(Field::Float(1.26), short_field(b"1.26  "));
        assert_done(Field::Float(1.), short_field(b" 1. "));
        assert_done(Field::Float(2.), short_field(b" 2."));
        assert_done(Field::Float(3.), short_field(b"3."));
        assert_done(Field::Float(4.), short_field(b"4. "));
        assert_done(Field::Float(1.23e7), short_field(b"1.23e+7"));
        assert_done(Field::Float(1.24e7), short_field(b"1.24e+7 "));
        assert_done(Field::Float(2.0e7), short_field(b"2e+7 "));
        assert_done(Field::Float(1.25e7), short_field(b"1.25+7"));
        assert_done(Field::Float(1.26e7), short_field(b"1.26+7 "));
        assert_done(Field::Float(1.0e7), short_field(b"1.+7 "));
        assert_done(Field::Int(123456), short_field(b"123456"));
        assert_done(Field::Continuation(b"A B"), short_field_cont(b"+A B"));
        assert_done(Field::String(b"HI1"), short_field(b"HI1"));
        assert_error(error_code!(ErrorKind::Alt), short_field(b"ABCDEFGHIJ"));
        assert_error(error_code!(ErrorKind::Alt), short_field(b"ABCDEFGHI"));
        assert_done(Field::String(b"ABCDEFGH"), short_field(b"ABCDEFGH"));
    }
}
