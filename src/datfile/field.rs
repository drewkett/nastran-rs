
use std::str;

use super::{Field, Card};

use nom::{self, IResult, Slice, InputIter, is_space, is_alphanumeric, is_alphabetic, is_digit, digit};
use errors;

fn parse_nastran_float(value: &[u8], exponent: &[u8]) -> f32 {
    let length = value.len() + exponent.len() + 1;
    let mut temp = Vec::with_capacity(length);
    for &c in value {
        temp.push(c);
    }
    temp.push(b'e');
    for &c in exponent {
        temp.push(c);
    }
    String::from_utf8_lossy(&temp[..]).parse::<f32>().expect("Failed to parse nastran float")
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

#[inline]
fn count_spaces(buffer: &[u8]) -> usize {
    return buffer.iter().take_while(|&&c| c == b' ').count();
}

#[inline]
fn trim_spaces(buffer: &[u8]) -> &[u8] {
    let n = buffer.len();
    let i = buffer.iter().take_while(|&&c| c == b' ').count();
    let j = buffer.iter()
        .skip(i)
        .rev()
        .take_while(|&&c| c == b' ')
        .count();
    &buffer[i..n - j]
}

#[inline]
fn is_plus_minus(c: u8) -> bool {
    c == b'+' || c == b'-'
}

#[inline]
fn is_alpha(c: u8) -> bool {
    (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z')
}

#[inline]
fn is_numeric(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

#[inline]
fn count_digits(buffer: &[u8]) -> usize {
    buffer.iter().take_while(|&&c| is_numeric(c)).count()
}

#[inline]
fn count_alphanumeric(buffer: &[u8]) -> usize {
    buffer.iter().take_while(|&&c| is_numeric(c) || is_alpha(c)).count()
}

fn print_slice(s: &str, buffer: &[u8]) {
    let b = unsafe { str::from_utf8_unchecked(buffer) };
    println!("{} {}",s,b)
}

fn maybe_string(buffer: &[u8]) -> errors::Result<Field> {
    let n = buffer.len();
    if n == 0 {
        return Err(errors::ErrorKind::ParseFailure.into());
    } else if n > 8 {
        return Err(errors::ErrorKind::ParseFailure.into());
    }
    if !is_alpha(buffer[0]) {
        return Err(errors::ErrorKind::ParseFailure.into());
    }
    let mut i = 1;
    i += count_alphanumeric(&buffer[i..]);
    let j = i;
    i += count_spaces(&buffer[i..]);
    if i == n {
        return Ok(Field::String(buffer));
    }
    if buffer[i] == b'*' {
        i += 1;
        if i == n {
            return Ok(Field::DoubleString(&buffer[..j]));
        }
    }
    return Err(errors::ErrorKind::ParseFailure.into());
}

fn maybe_number(buffer: &[u8]) -> errors::Result<Field> {
    let n = buffer.len();
    let mut i = 0;
    if is_plus_minus(buffer[i]) {
        i += 1
    }
    if i == n {
        return Err(errors::ErrorKind::ParseFailure.into());
    }
    let mut try_read_exponent = false;
    if is_numeric(buffer[i]) {
        i += count_digits(&buffer[i..]);
        if i == n {
            if i <= 8 {
                let s = unsafe { str::from_utf8_unchecked(buffer) };
                return s.parse().map(|v| Field::Int(v)).map_err(|e| e.into());
            } else {
                return Err(errors::ErrorKind::ParseFailure.into());
            }
        } else if buffer[i] == b'.' {
            i += 1;
            i += count_digits(&buffer[i..]);
            if i == n {
                let s = unsafe { str::from_utf8_unchecked(buffer) };
                return s.parse().map(|v| Field::Float(v)).map_err(|e| e.into());
            }
        }
        try_read_exponent = true;
    } else if buffer[i] == b'.' {
        i += 1;
        let n_digits = count_digits(&buffer[i..]);
        if n_digits == 0 {
            return Err(errors::ErrorKind::ParseFailure.into());
        }
        i += n_digits;
        try_read_exponent = true;
        if i == n {
            let s = unsafe { str::from_utf8_unchecked(buffer) };
            return s.parse().map(|v| Field::Float(v)).map_err(|e| e.into());
        }
    }
    if try_read_exponent {
        if buffer[i] == b'e' || buffer[i] == b'E' {
            i += 1;
            if i == n {
                return Err(errors::ErrorKind::ParseFailure.into());
            }
            if is_plus_minus(buffer[i]) {
                i += 1;
                if i == n {
                    return Err(errors::ErrorKind::ParseFailure.into());
                }
            }
            let n_digits = count_digits(&buffer[i..]);
            if n_digits == 0 || i + n_digits != n {
                return Err(errors::ErrorKind::ParseFailure.into());
            }
            let s = unsafe { str::from_utf8_unchecked(buffer) };
            return s.parse().map(|v| Field::Float(v)).map_err(|e| e.into());
        } else if buffer[i] == b'+' || buffer[i] == b'-' {
            let j = i;
            i += 1;
            if i == n {
                return Err(errors::ErrorKind::ParseFailure.into());
            }
            let n_digits = count_digits(&buffer[i..]);
            if n_digits == 0 || i + n_digits != n {
                return Err(errors::ErrorKind::ParseFailure.into());
            }
            let mut temp = [b' '; 80];
            temp[..j].copy_from_slice(&buffer[..j]);
            temp[j] = b'e';
            temp[j + 1..n + 1].copy_from_slice(&buffer[j..]);
            let s = unsafe { str::from_utf8_unchecked(&temp[..n + 1]) };
            return s.parse().map(|v| Field::Float(v)).map_err(|e| e.into());
        }
    }
    return Err(errors::ErrorKind::ParseFailure.into());
}

fn maybe_field(buffer: &[u8]) -> errors::Result<Field> {
    let n = buffer.len();
    if n == 0 {
        return Ok(Field::Blank);
    }
    if buffer[0] == b'+' {
        if n > 1 && (is_numeric(buffer[1]) || buffer[1] == b'.') {
            return maybe_number(trim_spaces(buffer));
        } else if n < 8 {
            return Ok(Field::Continuation(&buffer[1..]));
        } else {
            return Err(errors::ErrorKind::ParseFailure.into());
        }
    } else if buffer[0] == b'*' {
        if n < 8 {
            return Ok(Field::DoubleContinuation(&buffer[1..]));
        } else {
            return Err(errors::ErrorKind::ParseFailure.into());
        }
    }
    let buffer = trim_spaces(buffer);
    if buffer.len() == 0 {
        return Ok(Field::Blank);
    }
    match buffer[0] {
        b'a'...b'z' | b'A'...b'Z' => return maybe_string(buffer),
        b'+' | b'-' | b'0'...b'9' | b'.' => return maybe_number(buffer),
        _ => return Err(errors::ErrorKind::ParseFailure.into()),
    }
}

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

    #[bench]
    fn bench_maybe_field_nastran_float(b: &mut Bencher) {
        b.iter(|| maybe_field(b"11.22+7"));
    }

    #[bench]
    fn bench_maybe_field_float(b: &mut Bencher) {
        b.iter(|| maybe_field(b"11.22e+7"));
    }

    fn assert_done<T: Debug + PartialEq>(result: T, test: IResult<&[u8], T>) {
        assert_eq!(IResult::Done(&b""[..],result),test);
    }

    fn assert_error<T: Debug + PartialEq>(result: Err<u32>, test: IResult<&[u8], T>) {
        assert_eq!(IResult::Error(result),test);
    }

    fn success_maybe_field(test: &str, result: Field) {
        match maybe_field(test.as_bytes()) {
            Ok(r) => assert_eq!(r, result),
            Err(_) => panic!("Expected Ok for '{}'",test),
        }
    }

    #[test]
    fn test_parse() {
        success_maybe_field("+A B", Field::Continuation(b"A B"));
        success_maybe_field("+", Field::Continuation(b""));
        success_maybe_field("HI1", Field::String(b"HI1"));
        success_maybe_field("ABCDEFGH", Field::String(b"ABCDEFGH"));
        success_maybe_field(" 2.23 ", Field::Float(2.23));
        success_maybe_field("+2.24 ", Field::Float(2.24));
        success_maybe_field(" 2.25e7 ", Field::Float(2.25e7));
        success_maybe_field(" 2.26e+7 ", Field::Float(2.26e7));
        success_maybe_field(" 2.27e-7 ", Field::Float(2.27e-7));
        success_maybe_field(" .28 ", Field::Float(0.28));
        success_maybe_field(" .29e+7 ", Field::Float(0.29e7));
        success_maybe_field(" 30e+7 ", Field::Float(3e8));
        success_maybe_field(" 3.1+7 ", Field::Float(3.1e7));
        success_maybe_field(" 3.+7 ", Field::Float(3.0e7));
        success_maybe_field(" .2+7 ", Field::Float(0.2e7));
        success_maybe_field(" .2-7 ", Field::Float(0.2e-7));
        success_maybe_field("HI2*", Field::DoubleString(b"HI2"));
        success_maybe_field("HI3 *", Field::DoubleString(b"HI3"));
        success_maybe_field("* HI4", Field::DoubleContinuation(b" HI4"));
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
