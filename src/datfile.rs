
use std::cmp::min;
use std::fmt;
use std::str;

use dtoa;
use nom;
use nom::{Slice, digit, IResult, is_digit, is_alphanumeric, InputIter,
          is_alphabetic, is_space, rest};

use errors::{Result, ErrorKind};


#[derive(PartialEq)]
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
        if n > 0 && buf[n - 1] == b'0' {
            unsafe { String::from_utf8_unchecked(buf[..n - 1].to_vec()) }
        } else if n > 0 && buf[0] == b'0' {
            unsafe { String::from_utf8_unchecked(buf[1..n].to_vec()) }
        } else {
            format!("{:8e}", f)
        }
    } else {
        format!("{:8e}", f)
    }
}

fn double_to_8(f: f64) -> String {
    let mut buf = [b' '; 9];
    if let Ok(n) = dtoa::write(&mut buf[..], f) {
        if n > 0 && buf[0] == b'0' {
            unsafe { String::from_utf8_unchecked(buf[1..n].to_vec()) }
        } else if n > 0 && buf[n - 1] == b'0' {
            unsafe { String::from_utf8_unchecked(buf[..n - 1].to_vec()) }
        } else {
            format!("{:8e}", f)
        }
    } else {
        format!("{:8e}", f)
    }

}

impl<'a> fmt::Debug for Field<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Field::Blank => write!(f, "Blank"),
            Field::Int(i) => write!(f, "Int({})", i),
            Field::Float(d) => write!(f, "Float({})", d),
            Field::Double(d) => write!(f, "Double({})", d),
            Field::Continuation(c) => {
                write!(f,
                       "Continuation('{}')",
                       unsafe { str::from_utf8_unchecked(c) })
            }
            Field::String(s) => write!(f, "String('{}')", unsafe { str::from_utf8_unchecked(s) }),
        }
    }
}
impl<'a> fmt::Display for Field<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Field::Blank => write!(f, "        "),
            Field::Int(i) => write!(f, "{:8}", i),
            Field::Float(d) => write!(f, "{:>8}", float_to_8(d)),
            Field::Double(d) => write!(f, "{:>8}", double_to_8(d)),
            Field::Continuation(c) => write!(f, "+{:7}", unsafe { str::from_utf8_unchecked(c) }),
            Field::String(s) => write!(f, "{:8}", unsafe { str::from_utf8_unchecked(s) }),
        }
    }
}

#[derive(PartialEq)]
pub struct Card<'a> {
    pub fields: Vec<Field<'a>>,
    pub comment: Option<&'a [u8]>,
    pub is_double: bool,
    pub is_comma: bool,
    pub unparsed: Option<&'a [u8]>,
}

impl<'a> Card<'a> {
    fn from_first_field(first_field: Field<'a>, is_double: bool) -> Card<'a> {
        Card {
            fields: vec![first_field],
            is_comma: false,
            comment: None,
            unparsed: None,
            is_double,
        }
    }
}

impl<'a> fmt::Debug for Card<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "Card("));
        for field in &self.fields {
            try!(write!(f, "{:?},", field));
        }
        if let Some(comment) = self.comment {
            try!(write!(f,
                        "Comment='{}',",
                        unsafe { str::from_utf8_unchecked(comment) }));
        }
        if self.is_comma {
            try!(write!(f, "comma,"));
        }
        if self.is_double {
            try!(write!(f, "double,"));
        }
        if let Some(unparsed) = self.unparsed {
            try!(write!(f,
                        "Unparsed='{}',",
                        unsafe { str::from_utf8_unchecked(unparsed) }));
        }
        write!(f, ")")
    }
}


impl<'a> fmt::Display for Card<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for field in &self.fields {
            try!(write!(f, "{}", field));
        }
        if let Some(comment) = self.comment {
            try!(write!(f, "${}", unsafe { str::from_utf8_unchecked(comment) }));
        }
        write!(f, "")
    }
}

#[derive(Debug,PartialEq)]
pub struct Deck<'a> {
    pub cards: Vec<Card<'a>>,
}

impl<'a> fmt::Display for Deck<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "Deck(\n"));
        for card in &self.cards {
            try!(write!(f, "  {},\n", card));
        }
        write!(f, ")")
    }
}

fn read_first_field(line: &[u8]) -> IResult<&[u8], Card> {
    let mut is_comma = false;
    let length = line.len();
    let size = min(length, 8);
    let mut i_end = size;
    let mut consume_next = false;
    for (i, &c) in line.iter().take(8).enumerate() {
        if c == b',' {
            is_comma = true;
            consume_next = true;
            i_end = i;
            break;
        } else if c == b'\t' {
            i_end = i;
            consume_next = true;
            break;
        }
    }
    if i_end == size && length > 8 && line[8] == b',' {
        is_comma = true;
        consume_next = true;
    }
    let (line, mut remainder) = line.split_at(i_end);
    if consume_next {
        remainder = &remainder[1..];
    }
    let (_, mut card) = try_parse!(line, first_field);
    card.is_comma = is_comma;
    IResult::Done(remainder, card)
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
    String::from_utf8_lossy(&temp[..])
               .parse::<f32>()
               .expect("Failed to parse nastran float")
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
named!(field_float<Field>,map!(my_float, |f| Field::Float(f)));
named!(field_double<Field>,map!(my_double, |f| Field::Double(f)));

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

named!(short_field<Field>,
    alt_complete!(
        pad_space_eof!(field_float) |
        pad_space_eof!(field_nastran_float) |
        pad_space_eof!(field_integer) |
        pad_space_eof!(field_string) |
        value!(Field::Blank,tuple!(take_while!(is_space),eof!())
    )
));

named!(short_field_cont<Field>,
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

named!(first_field<Card>, alt_complete!(
    map!(pad_space_eof!(field_string),|field| Card::from_first_field(field,false)) |
    map!(pad_space_eof!(field_string_double),|field| Card::from_first_field(field,true)) |
    map!(terminated!(field_cont_single,eof!()),|field| Card::from_first_field(field,false)) |
    map!(terminated!(field_cont_double,eof!()),|field| Card::from_first_field(field,true)) |
    value!(Card::from_first_field(Field::Blank,false),tuple!(take_while!(is_space),eof!()))
));

named!(field_8<Field>, flat_map!(take_m_n_while!(0,8,move |c| c!= b'\t'),short_field));
named!(field_8_cont<Field>, flat_map!(take_m_n_while!(0,8,move |c| c!= b'\t'),field_cont));
named!(field_16<Field>, flat_map!(take_m_n_while!(0,16,move |c| c!= b'\t'),long_field));

fn option_from_slice(sl: &[u8]) -> Option<&[u8]> {
    if !sl.is_empty() { Some(sl) } else { None }
}

named!(split_short_with_cont<(Vec<Field>,Option<&[u8]>)>, do_parse!(
    fields: many_m_n!(8,8,field_8) >>
    last_field: opt!(field_8_cont) >>
    take_while!(is_space) >>
    unparsed: map!(rest,option_from_slice) >>
    ({
        let mut mfields = fields;
        if let Some(f) = last_field { mfields.push(f) } ;
        (mfields, unparsed)
    })
));
named!(split_short_partial<(Vec<Field>,Option<&[u8]>)>, do_parse!(
    fields: many_m_n!(0,7,field_8) >>
    (fields, None)
));
named!(split_short<(Vec<Field>,Option<&[u8]>)>,alt_complete!(
    split_short_with_cont|split_short_partial
    ));

named!(split_long_with_cont<(Vec<Field>,Option<&[u8]>)>, do_parse!(
    fields: many_m_n!(4,4,field_16) >>
    last_field: opt!(field_8_cont) >>
    take_while!(is_space) >>
    unparsed: map!(rest,option_from_slice) >>
    ({
        let mut mfields = fields;
        if let Some(f) = last_field { mfields.push(f) } ;
        (mfields, unparsed)
    })
));

named!(split_long_partial<(Vec<Field>,Option<&[u8]>)>, do_parse!(
    fields: many_m_n!(0,3,field_16) >>
    (fields, None)
));

named!(split_long<(Vec<Field>,Option<&[u8]>)>,alt_complete!(
    split_long_with_cont|split_long_partial));


fn split_line(line: &[u8]) -> IResult<&[u8], Card> {
    if line.is_empty() {
        return IResult::Done(b"",
                             Card {
                                 fields: vec![],
                                 is_comma: false,
                                 is_double: false,
                                 comment: None,
                                 unparsed: None,
                             });
    }
    let (mut remainder, mut card) = try_parse!(line, read_first_field);
    if card.is_comma {
        let mut i = 2;
        for sl in remainder.split(|&b| b == b',') {
            if i % 10 == 0 || i % 10 == 1 {
                let (_, field) = try_parse!(sl, short_field_cont);
                card.fields.push(field);
            } else {
                let (_, field) = try_parse!(sl, short_field);
                card.fields.push(field);
            }
            i += 1;
        }
        remainder = b"";
    } else if card.is_double {
        let (new_remainder, (fields, unparsed)) = try_parse!(remainder, split_long);
        card.fields.extend(fields);
        card.unparsed = unparsed;
        remainder = new_remainder;
    } else {
        let (new_remainder, (fields, unparsed)) = try_parse!(remainder, split_short);
        card.fields.extend(fields);
        card.unparsed = unparsed;
        remainder = new_remainder;
    }
    IResult::Done(remainder, card)
}

named!(split_line_nom<Card>,map!(
    tuple!(
        flat_map!(take_m_n_while!(0,80,call!(|c| c != b'$' && c != b'\n')),split_line),
        alt!(
            map!(alt!(eof!()|tag!("\n")),|_| None) |
            map!(preceded!(opt!(tag!("$")),take_until_and_consume!("\n")),|c| Some(c))
        )
    )
,|(card,comment)| Card { comment, .. card}));

named!(split_lines_nom<Deck>,map!(complete!(many0!(split_line_nom)),|cards| Deck { cards }));

pub fn parse_buffer(buffer: &[u8]) -> Result<Deck> {
    match split_lines_nom(buffer) {
        IResult::Done(_, d) => Ok(d),
        IResult::Error(_) | IResult::Incomplete(_) => Err(ErrorKind::ParseFailure.into()),
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use test::Bencher;

    use super::*;

    #[bench]
    fn bench_field_nastran_float(b: &mut Bencher) {
        b.iter(|| field_nastran_float(b"11.22+7"));
    }

    #[bench]
    fn bench_field_float(b: &mut Bencher) {
        b.iter(|| field_float(b"11.22e+7"));
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
