use std::cmp::min;
use std::iter::Peekable;
use std::slice::Iter;
use std::fmt;
use std::io::{Result, Error, ErrorKind};
use regex::bytes::Regex;

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
        let mut i_comment = 80;
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
    lazy_static! {
        static ref STRING: Regex = Regex::new(r"^[a-zA-Z]+$").unwrap();
    }
    let first_slice = &line[..i_end];
    let first_field = if STRING.is_match(first_slice) {
        Field::String(String::from_utf8_lossy(first_slice).into_owned())
    } else {
        return Err(Error::new(ErrorKind::Other,"Invalid first field"))
    };
    let remainder = &line[i_next..];
    return Ok((first_field, flags, remainder));
}

fn strip_spaces(field: &[u8]) -> &[u8] {
    let length = field.len();
    let mut i_start = 0;
    for i in 0..length {
        if field[i] == b' ' {
            i_start = i + 1;
        } else {
            break;
        }
    }
    let field = &field[i_start..];
    if field.len() == 0 {
        return field;
    }
    let length = field.len();
    let mut i_end = length;
    for i in length - 1..0 {
        if field[i] == b' ' {
            i_end = i;
        } else {
            break;
        }
    }
    return &field[..i_end];
}

fn parse_field_as_string(field: &[u8]) -> Result<Field> {
    for i in 0..field.len() {
        match field[i] {
            b'a'...b'z' | b'A'...b'Z' | b'0'...b'9' => (),
            _ => return Err(Error::new(ErrorKind::Other, "Invalid character in field")),
        }
    }
    return Ok(Field::String(String::from_utf8_lossy(field).into_owned()));
}

fn parse_field_as_float(field: &[u8]) -> Result<Field> {
    let length = field.len();
    if length == 0 {
        return Ok(Field::Blank);
    }
    lazy_static! {
        static ref STRING: Regex = Regex::new(r"^[a-zA-Z]+$").unwrap();
    }
    if STRING.is_match(field) {
        return Ok(Field::String(String::from_utf8_lossy(field).into_owned())) 
    } else {
        return Err(Error::new(ErrorKind::Other, "Invalid String"))
    }
}

fn parse_field_as_number(field: &[u8]) -> Result<Field> {
    let length = field.len();
    if length == 0 {
        return Ok(Field::Blank);
    }
    lazy_static! {
        static ref INT: Regex = Regex::new(r"^-?[0-9]+$").unwrap();
        static ref FLOAT: Regex = Regex::new(r"^[-+]?([0-9]+\.[0-9]*|\.[0-9]+)(([eE][-+]?|[-+])[0-9]+)?$").unwrap();
    }
    if INT.is_match(field) {
        let number: i32 = String::from_utf8_lossy(field).parse().unwrap();
        return Ok(Field::Int(number))
    } else if FLOAT.is_match(field) {
        let number: f32 = String::from_utf8_lossy(field).parse().unwrap();
        return Ok(Field::Float(number))
    } else {
        return Err(Error::new(ErrorKind::Other, "Can't parse number"))
    }
}

fn parse_field(field: &[u8]) -> Result<Field> {
    let field = strip_spaces(field);
    if field.len() == 0 {
        return Ok(Field::Blank);
    }
    return match field[0] {
               b'a'...b'z' | b'A'...b'Z' => parse_field_as_string(field),
               b'0'...b'9' | b'-' | b'+' | b'.' => parse_field_as_number(field),
               _ => Err(Error::new(ErrorKind::Other, "Can't parse field")),
           };
}

fn split_line(line: &[u8]) -> Result<Vec<Field>> {
    let (field, flags, remainder) = match read_first_field(line) {
        Ok(r) => r,
        Err(e) => return Err(e)
    };
    let mut fields = vec![field];
    if flags.is_comma {
        let it = remainder
            .split(|&b| b == b',')
            .map(parse_field);
        for f in it {
            match f {
                Ok(field) => fields.push(field),
                Err(e) => return Err(e)
            }
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
            Err(e) => return Err(e)
        };
        let comment = Some(line.comment.to_owned());
        cards.push(Card { fields, comment })
    }
    return Ok(Deck { cards });
}

