
use std::cmp::min;
use std::fmt;
use std::io::{Result, Error, ErrorKind};
use std::str;

use nom;
use nom::{Slice, digit, IResult, alphanumeric, is_digit, is_alphanumeric, InputIter, is_alphabetic};

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
pub enum Field {
    Blank,
    Int(i32),
    Float(f32),
    Double(f64),
    Continuation(String),
    String(String),
}

struct FlaggedField {
    field: Field,
    flags: CardFlags
}

fn string_field_from_slice(b: &[u8]) -> IResult<&[u8], Field> {
    match str::from_utf8(b) {
        Ok(s) => IResult::Done(b"",Field::String(s.to_owned())),
        Err(_) => IResult::Error(error_code!(nom::ErrorKind::Custom(102)))
    }
}

fn cont_field_from_slice(b: &[u8]) -> IResult<&[u8], Field> {
    match str::from_utf8(b) {
        Ok(s) => IResult::Done(b"",Field::Continuation(s.to_owned())),
        Err(_) => IResult::Error(error_code!(nom::ErrorKind::Custom(102)))
    }
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

fn parse_first_field(field_slice: &[u8]) -> Result<FlaggedField> {
    return match first_field(field_slice) {
        IResult::Done(_,f) => Ok(f),
        _ => Err(Error::new(ErrorKind::Other, format!("Can't parse field '{}'",String::from_utf8_lossy(field_slice)))),
    };
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
    let flagged_field = match parse_first_field(&line[..i_end]) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };
    let field = flagged_field.field;
    flags.is_double = flagged_field.flags.is_double;
    let remainder = &line[i_next..];
    return Ok((field, flags, remainder));
}


pub fn parse_short_field(field: &[u8]) -> Result<Field> {
    return match short_field(field) {
        IResult::Done(_,f) => Ok(f),
        _ => Err(Error::new(ErrorKind::Other, format!("Can't parse field '{}'",String::from_utf8_lossy(field)))),
    };
}

pub fn parse_short_field_cont(field: &[u8]) -> Result<Field> {
    return match short_field_cont(field) {
        IResult::Done(_,f) => Ok(f),
        _ => Err(Error::new(ErrorKind::Other, format!("Can't parse field '{}'",String::from_utf8_lossy(field)))),
    };
}

pub fn parse_long_field(field: &[u8]) -> Result<Field> {
    return match long_field(field) {
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
    return String::from_utf8_lossy(&temp[..]).parse::<f32>().expect("Failed to parse nastran float");
}

named!(field_string<Field>,flat_map!(recognize!(tuple!(char_if!(is_alphabetic),take_m_n_while!(0,7,is_alphanumeric))),string_field_from_slice));
named!(field_string_double<Field>,flat_map!(
    terminated!(
        recognize!( tuple!(char_if!(is_alphabetic),take_m_n_while!(0,7,is_alphanumeric)))
        ,tuple!(many0!(tag!(" ")),tag!("*"))
    ), string_field_from_slice));
named!(field_cont<Field>,flat_map!(preceded!(tag!("+"),recognize!(many0!(alt!(tag!(" ")|alphanumeric)))),cont_field_from_slice));
named!(field_cont_double<Field>,flat_map!(preceded!(tag!("*"),recognize!(many0!(alt!(tag!(" ")|alphanumeric)))),cont_field_from_slice));
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
            terminated!(field_cont,eof!()) |
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
            map!(terminated!(field_cont,eof!()),|field| FlaggedField {field, flags: CardFlags { is_double: false, is_comma: false }}) |
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


fn split_line(line: &[u8]) -> Result<Vec<Field>> {
    let (field, flags, remainder) = match read_first_field(line) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let mut fields = vec![field];
    if flags.is_comma {
        let it = remainder.split(|&b| b == b',').map(parse_short_field);
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
            } else if i == 9 {
                match parse_short_field_cont(field_slice) {
                    Ok(field) => fields.push(field),
                    Err(e) => return Err(e),
                }
            } else {
                match parse_short_field(field_slice) {
                    Ok(field) => fields.push(field),
                    Err(e) => return Err(e),
                }
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
