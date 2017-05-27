
use std::cmp::min;
use std::fmt;
use std::str;

use dtoa;
use nom;
use nom::{Slice, digit, IResult, alphanumeric, is_digit, is_alphanumeric, InputIter, is_alphabetic};

use errors::{Result,ErrorKind};

macro_rules! take_m_n_while (
  ($i:expr, $m:expr, $n: expr, $submac:ident!( $($args:tt)* )) => (
      {
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



#[derive(Debug,PartialEq)]
pub enum Field<'a> {
    Blank,
    Int(i32),
    Float(f32),
    Double(f64),
    Continuation(&'a [u8]),
    String(&'a [u8]),
}

fn float_to_8(f: f32) -> String {
    let mut buf = [b' '; 9];
    if let Ok(n) = dtoa::write(&mut buf[..], f) {
            if n > 0 && buf[0] == b'0' {
                unsafe { String::from_utf8_unchecked(buf[1..n].to_vec()) }
            } else if n > 0 && buf[n-1] == b'0' {
                unsafe { String::from_utf8_unchecked(buf[..n-1].to_vec()) }
            } else {
                format!("{:8e}",f)
            }
    } else {
        format!("{:8e}",f)
    }
}

fn double_to_8(f: f64) -> String {
    let mut buf = [b' '; 9];
    if let Ok(n) = dtoa::write(&mut buf[..], f) {
            if n > 0 && buf[0] == b'0' {
                unsafe { String::from_utf8_unchecked(buf[1..n].to_vec()) }
            } else if n > 0 && buf[n-1] == b'0' {
                unsafe { String::from_utf8_unchecked(buf[..n-1].to_vec()) }
            } else {
                format!("{:8e}",f)
            }
    } else {
        format!("{:8e}",f)
    }

}

impl <'a> fmt::Display for Field<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Field::Blank => write!(f,"        "),
            &Field::Int(i) => write!(f,"{:8}",i),
            &Field::Float(d) => write!(f,"{:>8}",float_to_8(d)),
            &Field::Double(d) => write!(f,"{:>8}",double_to_8(d)),
            &Field::Continuation(c) => write!(f,"+{:7}",unsafe {str::from_utf8_unchecked(c)}),
            &Field::String(s) => write!(f,"{:8}",unsafe {str::from_utf8_unchecked(s)}),
        }
    }
}

struct FlaggedField<'a> {
    field: Field<'a>,
    flags: CardFlags
}

#[derive(Debug,PartialEq)]
pub struct CardFlags {
    is_double: bool,
    is_comma: bool,
}

#[derive(PartialEq)]
pub struct Card <'a> {
    pub fields: Vec<Field<'a>>,
    pub comment: Option<&'a [u8]>,
}

impl <'a> fmt::Debug for Card<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "Card("));
        for field in self.fields.iter() {
            try!(write!(f, "{:?},", field));
        }
        if let Some(comment) = self.comment {
            try!(write!(f, "Comment='{}'", unsafe { str::from_utf8_unchecked(comment)}));
        }
        write!(f, ")")
    }
}

impl <'a> fmt::Display for Card<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for field in self.fields.iter() {
            try!(write!(f, "{}", field));
        }
        if let Some(comment) = self.comment {
            try!(write!(f, "${}", unsafe { str::from_utf8_unchecked(comment)}));
        }
        write!(f,"")
    }
}

#[derive(Debug,PartialEq)]
pub struct Deck<'a> {
    pub cards: Vec<Card<'a>>,
}

impl <'a> fmt::Display for Deck<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "Deck(\n"));
        for card in self.cards.iter() {
            try!(write!(f, "  {},\n", card));
        }
        write!(f, ")")
    }
}

fn read_first_field(line: &[u8]) -> IResult<&[u8],(Field, CardFlags)> {
    let mut flags = CardFlags {
        is_comma: false,
        is_double: false,
    };
    let length = line.len();
    let size = min(length, 8);
    let mut i_end = size;
    let mut i_next = size;
    for i in 0..size {
        if line[i] == b',' {
            flags.is_comma = true;
            i_end = i;
            i_next = i + 1;
            break;
        } else if line[i] == b'\t' {
            i_end = i;
            i_next = i + 1;
            break;
        }
    }
    if i_end == size && length > 8 {
        if line[8] == b',' {
            flags.is_comma = true;
            i_next = 9;
        }
    }
    let flagged_field = match first_field(&line[..i_end]) {
        IResult::Done(_,res) => res,
        IResult::Error(e) => return IResult::Error(e),
        _ => unreachable!()
    };
    let field = flagged_field.field;
    flags.is_double = flagged_field.flags.is_double;
    let remainder = &line[i_next..];
    return IResult::Done(remainder,(field,flags))
}


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
    return String::from_utf8_lossy(&temp[..]).parse::<f32>().expect("Failed to parse nastran float");
}

named!(field_string<Field>,map!(recognize!(tuple!(char_if!(is_alphabetic),take_m_n_while!(0,7,is_alphanumeric))),Field::String));
named!(field_string_double<Field>,map!(
    terminated!(
        recognize!( tuple!(char_if!(is_alphabetic),take_m_n_while!(0,7,is_alphanumeric)))
        ,tuple!(many0!(tag!(" ")),tag!("*"))
    ), Field::String));
named!(field_cont_single<Field>,map!(preceded!(tag!("+"),recognize!(many0!(alt!(tag!(" ")|alphanumeric)))),Field::Continuation));
named!(field_cont_double<Field>,map!(preceded!(tag!("*"),recognize!(many0!(alt!(tag!(" ")|alphanumeric)))),Field::Continuation));
named!(field_cont_anon<Field>,map!(recognize!(many0!(alt!(tag!(" ")|alphanumeric))),Field::Continuation));
named!(field_cont<Field>,alt!(field_cont_single|field_cont_double|field_cont_anon));
named!(field_float<Field>,map!(my_float, |f| Field::Float(f)));
named!(field_double<Field>,map!(my_double, |f| Field::Double(f)));

named!(decimal_float_value, recognize!(alt!(
          delimited!(digit, tag!("."), opt!(complete!(digit)))
          | terminated!(tag!("."), digit)
        )
));

named!(float_exponent, recognize!(tuple!(
          alt!(tag!("e") | tag!("E")),
          opt!(alt!(tag!("+") | tag!("-"))),
          digit
          )
));

named!(double_exponent, recognize!(tuple!(
          alt!(tag!("d") | tag!("D")),
          opt!(alt!(tag!("+") | tag!("-"))),
          digit
          )
));

fn my_float(input: &[u8]) -> IResult<&[u8],f32> {
  flat_map!(input,
    recognize!(
      tuple!(
        opt!(alt!(tag!("+") | tag!("-"))),
        alt!(
            terminated!(decimal_float_value,opt!(complete!(float_exponent))) |
            terminated!(digit,float_exponent)
        )
      )
    ),
    parse_to!(f32)
  )
}

fn my_double(input: &[u8]) -> IResult<&[u8],f64> {
  flat_map!(input,
    recognize!(
      tuple!(
        opt!(alt!(tag!("+") | tag!("-"))),
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
                opt!(alt!(tag!("+") | tag!("-"))),
alt!(
          delimited!(digit, tag!("."), opt!(digit))
          | terminated!(tag!("."), digit)
        )

            )
        ),
        recognize!(tuple!(
            alt!(tag!("+") | tag!("-")),
            digit
        ))
        ),
        |(value,exponent)| Field::Float(parse_nastran_float(value,exponent))
    )
);

named!(field_integer<Field>,map!(flat_map!(
        recognize!(
            alt!(
                preceded!(tag!("-"),take_m_n_while!(1,7,is_digit))|
                take_m_n_while!(1,8,is_digit)
            )
        ),
        parse_to!(i32))
    ,|i| Field::Int(i))
);

macro_rules! pad_space_eof(
  ($i:expr, $submac:ident!( $($args:tt)* )) => (
    delimited!($i, many0!(tag!(" ")),$submac!($($args)*),tuple!(many0!(tag!(" ")),eof!()))
  );
  ($i:expr, $f:expr) => (
    pad_space_eof!($i, call!($f));
  );
);

named!(short_field<Field>,
       alt_complete!(
           pad_space_eof!(field_float) |
           pad_space_eof!(field_nastran_float) |
           pad_space_eof!(field_integer) |
           pad_space_eof!(field_string) |
            value!(Field::Blank,terminated!(many0!(tag!(" ")),eof!()))
));

named!(short_field_cont<Field>,
       alt_complete!(
           pad_space_eof!(field_float) |
           pad_space_eof!(field_nastran_float) |
           pad_space_eof!(field_integer) |
           pad_space_eof!(field_string) |
            terminated!(field_cont_single,eof!()) |
            terminated!(field_cont_double,eof!()) |
            value!(Field::Blank,terminated!(many0!(tag!(" ")),eof!()))
));

named!(long_field<Field>,
       alt_complete!(
           pad_space_eof!(field_float) |
           pad_space_eof!(field_double) |
           pad_space_eof!(field_nastran_float) |
           pad_space_eof!(field_integer) |
           pad_space_eof!(field_string) |
            value!(Field::Blank,terminated!(many0!(tag!(" ")),eof!()))
));

named!(first_field<FlaggedField>,
       alt_complete!(
           map!(pad_space_eof!(field_string),|field| FlaggedField {field, flags: CardFlags { is_double: false, is_comma: false }}) |
           map!(pad_space_eof!(field_string_double),|field| FlaggedField {field, flags: CardFlags { is_double: true, is_comma: false }}) |
            map!(terminated!(field_cont_single,eof!()),|field| FlaggedField {field, flags: CardFlags { is_double: false, is_comma: false }}) |
            map!(terminated!(field_cont_double,eof!()),|field| FlaggedField {field, flags: CardFlags { is_double: true, is_comma: false }}) |
            value!(FlaggedField {field:Field::Blank, flags: CardFlags{is_double:false,is_comma:false}},terminated!(many0!(tag!(" ")),eof!()))
));

struct ShortCardIterator<'a> {
    remainder: &'a [u8],
}

impl<'a> ShortCardIterator<'a> {
    fn new(remainder: &'a [u8]) -> ShortCardIterator {
        return ShortCardIterator { remainder };
    }
}

impl<'a> Iterator for ShortCardIterator<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        let n = min(8, self.remainder.len());
        if n == 0 {
            return None;
        }
        for i in 0..n {
            if self.remainder[i] == b'\t' {
                let field = &self.remainder[..i];
                self.remainder = &self.remainder[i + 1..];
                return Some(field);
            }
        }
        let field = &self.remainder[..n];
        self.remainder = &self.remainder[n..];
        return Some(field);
    }
}

named!(field_8<Field>, flat_map!(take_m_n_while!(0,8,move |c| c!= b'\t'),short_field));
named!(field_8_cont<Field>, flat_map!(take_m_n_while!(0,8,move |c| c!= b'\t'),field_cont));
named!(field_16<Field>, flat_map!(take_m_n_while!(0,16,move |c| c!= b'\t'),long_field));

named!(split_short<Vec<Field>>,many_m_n!(0,9,field_8));
named!(split_long_with_cont<Vec<Field>>, do_parse!(
    fields: many_m_n!(4,4,field_16) >>
    last_field: opt!(field_8_cont) >>
    ({ 
        let mut mfields = fields;
        if let Some(f) = last_field { mfields.push(f) } ;
        mfields
    })
));

named!(split_long<Vec<Field>>,alt_complete!(split_long_with_cont|many_m_n!(0,3,field_16)));


fn split_line(line: &[u8]) -> IResult<&[u8],Vec<Field>> {
    if line.len() == 0 {
        return IResult::Done(b"",vec![])
    }
    let (field, flags, mut remainder) = match read_first_field(line) {
        IResult::Done(remainder,(field,flags)) => (field,flags,remainder),
        IResult::Error(e) => return IResult::Error(e),
        _ => unreachable!()
    };
    let mut fields = vec![field];
    if flags.is_comma {
        let mut i = 2;
        for sl in remainder.split(|&b| b == b',') {
            if i % 10 == 0 || i % 10 == 1 {
             match short_field_cont(sl) {
                IResult::Done(_,field) => fields.push(field),
                IResult::Error(e) => return IResult::Error(e),
                _ => unreachable!()

             }
           } else {
             match short_field(sl) {
                IResult::Done(_,field) => fields.push(field),
                IResult::Error(e) => return IResult::Error(e),
                _ => unreachable!()
             }
           }
           i += 1;
        }
        remainder = b"";
    } else if flags.is_double {
        match split_long(remainder) {
            IResult::Done(_,rem_fields) => fields.extend(rem_fields),
            IResult::Error(e) => return IResult::Error(e),
            _ => unreachable!()
        }
    } else {
        let mut it = ShortCardIterator::new(remainder);
        let mut i = 2;
        while let Some(field_slice) = it.next() {
            if i > 10 {
                break;
                // return Err(Error::new(ErrorKind::Other,format!("Too many fields found in line '{}'",String::from_utf8_lossy(line))))
            } else if i == 10 {
                match field_cont(field_slice) {
                    IResult::Done(_,field) => fields.push(field),
                    IResult::Error(e) => return IResult::Error(e),
                    _ => unreachable!()
                }
            } else {
                match short_field(field_slice) {
                    IResult::Done(_,field) => fields.push(field),
                    IResult::Error(e) => return IResult::Error(e),
                    _ => unreachable!()

                }
            }
            i += 1;
        }
        remainder = it.remainder;
    }
    return IResult::Done(remainder,fields);
}

named!(split_line_nom<Card>,map!(
    tuple!(
        dbg!(flat_map!(take_m_n_while!(0,80,call!(|c| c != b'$' && c != b'\n')),split_line)),
        alt!(
            map!(tag!("\n"),|_| None) | 
            map!(preceded!(opt!(tag!("$")),take_until_and_consume!("\n")),|c| Some(c))
        )
    )
,|(fields,comment)| Card { fields, comment}));

named!(split_lines_nom<Deck>,map!(complete!(many0!(split_line_nom)),|cards| Deck { cards }));

pub fn parse_buffer(buffer: &[u8]) -> Result<Deck> {
    match split_lines_nom(buffer) {
        IResult::Done(_,d) => Ok(d),
        IResult::Error(_) => Err(ErrorKind::ParseFailure.into()),
        IResult::Incomplete(_) => unreachable!()
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use test::Bencher;

    use super::*;

    #[bench]
    fn bench_field_nastran_float(b: &mut Bencher) {
        b.iter(|| {
            field_nastran_float(b"11.22+7")
        });
    }

    #[bench]
    fn bench_field_float(b: &mut Bencher) {
        b.iter(|| {
            field_float(b"11.22e+7")
        });
    }

    #[test]
    fn test_parse() {
        assert_eq!(IResult::Done(&b""[..],Field::Float(1.23)),short_field(b" 1.23 "));
        assert_eq!(IResult::Done(&b""[..],Field::Float(1.24)),short_field(b" 1.24"));
        assert_eq!(IResult::Done(&b""[..],Field::Float(1.25)),short_field(b"1.25"));
        assert_eq!(IResult::Error(nom::ErrorKind::Alt),short_field(b"1252341551"));
        assert_eq!(IResult::Done(&b""[..],Field::Float(1.26)),short_field(b"1.26  "));
        assert_eq!(IResult::Done(&b""[..],Field::Float(1.)),short_field(b" 1. "));
        assert_eq!(IResult::Done(&b""[..],Field::Float(2.)),short_field(b" 2."));
        assert_eq!(IResult::Done(&b""[..],Field::Float(3.)),short_field(b"3."));
        assert_eq!(IResult::Done(&b""[..],Field::Float(4.)),short_field(b"4. "));
        assert_eq!(IResult::Done(&b""[..],Field::Float(1.23e7)),short_field(b"1.23e+7"));
        assert_eq!(IResult::Done(&b""[..],Field::Float(1.24e7)),short_field(b"1.24e+7 "));
        assert_eq!(IResult::Done(&b""[..],Field::Float(2.0e7)),short_field(b"2e+7 "));
        assert_eq!(IResult::Done(&b""[..],Field::Float(1.25e7)),short_field(b"1.25+7"));
        assert_eq!(IResult::Done(&b""[..],Field::Float(1.26e7)),short_field(b"1.26+7 "));
        assert_eq!(IResult::Done(&b""[..],Field::Float(1.0e7)),short_field(b"1.+7 "));
        assert_eq!(IResult::Done(&b""[..],Field::Int(123456)),short_field(b"123456"));
        assert_eq!(IResult::Done(&b""[..],Field::Continuation(b"A B")),short_field_cont(b"+A B"));
        assert_eq!(IResult::Done(&b""[..],Field::String(b"HI1")),short_field(b"HI1"));
        assert_eq!(IResult::Error(nom::ErrorKind::Alt),short_field(b"ABCDEFGHIJ"));
        assert_eq!(IResult::Error(nom::ErrorKind::Alt),short_field(b"ABCDEFGHI"));
        assert_eq!(IResult::Done(&b""[..],Field::String(b"ABCDEFGH")),short_field(b"ABCDEFGH"));
    }
}
