
use std::cmp::min;
use std::fmt;
use std::io::{Result, Error, ErrorKind};
use regex::bytes::Regex;

use nom;
use nom::{Slice, digit, IResult, is_alphabetic, is_alphanumeric};

lazy_static! {
    static ref BLANK: Regex = Regex::new(r"^ *$").unwrap();
    static ref STRING: Regex = Regex::new(r"^ *([a-zA-Z][a-zA-Z0-9]*) *$").unwrap();
    static ref DSTRING: Regex = Regex::new(r"^ *([a-zA-Z][a-zA-Z0-9]*) *\* *$").unwrap();
    static ref CONT: Regex = Regex::new(r"^\+([a-zA-Z0-9 \.]*)$").unwrap();
    static ref DCONT: Regex = Regex::new(r"^\*([a-zA-Z0-9 \.]*)$").unwrap();
}

#[derive(Debug,PartialEq)]
pub enum Field {
    Blank,
    Int(i32),
    Float(f32),
    Double(f64),
    Continuation(String),
    String(String),
}

#[derive(Debug,PartialEq)]
pub struct CardFlags {
    is_double: bool,
    is_comma: bool,
}

#[derive(Debug,PartialEq)]
pub struct Card {
    pub fields: Vec<Field>,
    pub comment: Option<Vec<u8>>,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "Card("));
        for field in self.fields.iter() {
            try!(write!(f, "{:?},", field));
        }
        if let Some(ref c) = self.comment {
            try!(write!(f, "Comment='{}'", String::from_utf8_lossy(c)));
        }
        write!(f, ")")
    }
}

#[derive(Debug,PartialEq)]
pub struct Deck {
    pub cards: Vec<Card>,
}

impl fmt::Display for Deck {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "Deck(\n"));
        for card in self.cards.iter() {
            try!(write!(f, "  {},\n", card));
        }
        write!(f, ")")
    }
}

struct Line<'a> {
    buffer: &'a [u8],
    comment: &'a [u8],
}

impl<'a> fmt::Display for Line<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Line ('{}',Comment='{}')",
               String::from_utf8_lossy(self.buffer),
               String::from_utf8_lossy(self.comment))
    }
}

struct Lines<'a> {
    buffer: &'a [u8],
}

impl<'a> Lines<'a> {
    fn new(buffer: &'a [u8]) -> Lines<'a> {
        return Lines { buffer };
    }
}

impl<'a> Iterator for Lines<'a> {
    type Item = Line<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let length = self.buffer.len();
        if length == 0 {
            return None;
        }
        let mut i_comment = min(80, length);
        let mut i_end = length;
        let mut i_next = length;
        for i in 0..self.buffer.len() {
            let c = self.buffer[i];
            if c == b'$' && i < i_comment {
                i_comment = i;
            } else if c == b'\r' || c == b'\n' {
                i_end = i;
                if i_comment > i_end {
                    i_comment = i_end;
                }
                i_next = i + 1;
                for i in i..self.buffer.len() {
                    let c = self.buffer[i];
                    if c == b'\r' || c == b'\n' {
                        i_next = i + 1;
                    } else {
                        break;
                    }
                }
                break;
            }
        }
        let line = Line {
            buffer: &self.buffer[..i_comment],
            comment: &self.buffer[i_comment..i_end],
        };
        self.buffer = &self.buffer[i_next..];
        return Some(line);
    }
}

fn parse_first_field(field_slice: &[u8]) -> Result<(Field, bool)> {
    if BLANK.is_match(field_slice) {
        return Ok((Field::Blank, false));
    } else if STRING.is_match(field_slice) {
        let cap = STRING.captures(field_slice).unwrap();
        let s = String::from_utf8_lossy(&cap[1]).into_owned();
        return Ok((Field::String(s), false));
    } else if DSTRING.is_match(field_slice) {
        let cap = DSTRING.captures(field_slice).unwrap();
        let s = String::from_utf8_lossy(&cap[1]).into_owned();
        return Ok((Field::String(s), true));
    } else if CONT.is_match(field_slice) {
        let cap = CONT.captures(field_slice).unwrap();
        let s = match cap.get(1) {
            Some(c) => String::from_utf8_lossy(c.as_bytes()).into_owned(),
            None => "".to_owned(),
        };
        return Ok((Field::Continuation(s), false));
    } else if DCONT.is_match(field_slice) {
        let cap = DCONT.captures(field_slice).unwrap();
        let s = match cap.get(1) {
            Some(c) => String::from_utf8_lossy(c.as_bytes()).into_owned(),
            None => "".to_owned(),
        };
        return Ok((Field::Continuation(s), true));
    } else {
        return Err(Error::new(ErrorKind::Other,
                              format!("Invalid first field '{}'",
                                      String::from_utf8_lossy(field_slice))));
    }
}

fn read_first_field(line: &[u8]) -> Result<(Field, CardFlags, &[u8])> {
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
    let (field, is_double) = match parse_first_field(&line[..i_end]) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };
    flags.is_double = is_double;
    let remainder = &line[i_next..];
    return Ok((field, flags, remainder));
}


pub fn parse_field(field: &[u8]) -> Result<Field> {
    return match parse_field_nom(field) {
        IResult::Done(_,f) => Ok(f),
        _ => Err(Error::new(ErrorKind::Other, format!("Can't parse field '{}'",String::from_utf8_lossy(field)))),
    };
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
    return String::from_utf8_lossy(&temp[..]).parse::<f32>().unwrap();
}

fn field_string(input: &[u8]) -> IResult<&[u8], Field> {
    let input_length = input.len();
    if input_length == 0 {
        return IResult::Incomplete(nom::Needed::Unknown);
    }
    for (idx, &item) in input.iter().enumerate() {
        if idx == 0 {
            if !is_alphabetic(item) {
                return IResult::Error(error_position!(nom::ErrorKind::Custom(100), input));
            }
        } else if !is_alphanumeric(item) {
            let s = String::from_utf8_lossy(&input[0..idx]).into_owned();
            return IResult::Done(&input[idx..], Field::String(s));
        }
    }
    let s = String::from_utf8_lossy(input).into_owned();
    IResult::Done(&input[input_length..], Field::String(s))
}

fn field_cont(input: &[u8]) -> IResult<&[u8], Field> {
    let input_length = input.len();
    if input_length == 0 {
        return IResult::Incomplete(nom::Needed::Unknown);
    }
    for (idx, &item) in input.iter().enumerate() {
        if idx == 0 {
            if item != b'+' {
                return IResult::Error(error_position!(nom::ErrorKind::Custom(101), input));
            }
        } else if !(is_alphanumeric(item) || item == b' ') {
            let s = String::from_utf8_lossy(&input[1..idx]).into_owned();
            return IResult::Done(&input[idx..], Field::Continuation(s));
        }
    }
    let s = String::from_utf8_lossy(&input[1..]).into_owned();
    IResult::Done(&input[input_length..], Field::Continuation(s))
}

named!(field_float<Field>,map!(my_float, |f| Field::Float(f)));

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
            tuple!(
                opt!(tag!("-")),
                digit
            )
        ),
        parse_to!(i32))
    ,|i| Field::Int(i))
);

macro_rules! pad_space(
  ($i:expr, $submac:ident!( $($args:tt)* )) => (
    delimited!($i, many0!(tag!(" ")),$submac!($($args)*),tuple!(many0!(tag!(" ")),eof!()))
  );
  ($i:expr, $f:expr) => (
    pad_space!($i, call!($f));
  );
);

named!(parse_field_nom<Field>,
       alt_complete!(
           pad_space!(field_float) |
           pad_space!(field_nastran_float) |
           pad_space!(field_integer) |
           pad_space!(field_string) |
            terminated!(field_cont,eof!()) |
            value!(Field::Blank,terminated!(many0!(tag!(" ")),eof!()))
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


fn split_line(line: &[u8]) -> Result<Vec<Field>> {
    let (field, flags, remainder) = match read_first_field(line) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let mut fields = vec![field];
    if flags.is_comma {
        let it = remainder.split(|&b| b == b',').map(parse_field);
        for f in it {
            match f {
                Ok(field) => fields.push(field),
                Err(e) => return Err(e),
            }
        }
    } else if flags.is_double {
    } else {
        let mut it = ShortCardIterator::new(remainder);
        let mut i = 0;
        while let Some(field_slice) = it.next() {
            if i > 9 {
                break;
                // return Err(Error::new(ErrorKind::Other,format!("Too many fields found in line '{}'",String::from_utf8_lossy(line))))
            }
            match parse_field(field_slice) {
                Ok(field) => fields.push(field),
                Err(e) => return Err(e),
            }
            i += 1;
        }
    }
    return Ok(fields);
}

pub fn parse_buffer(buffer: &[u8]) -> Result<Deck> {
    let mut cards = vec![];
    let mut lines_it = Lines::new(buffer);
    while let Some(line) = lines_it.next() {
        let fields = match split_line(line.buffer) {
            Ok(fields) => fields,
            Err(e) => return Err(e),
        };
        let comment = Some(line.comment.to_owned());
        cards.push(Card { fields, comment })
    }
    return Ok(Deck { cards });
}
