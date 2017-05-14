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
    Continuation(Vec<u8>),
    String(Vec<u8>),
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

struct NastranIterator<'a> {
    iter: &'a mut Peekable<Iter<'a, u8>>,
    is_comma: bool,
    is_double: bool,
    field_index: usize,
    line_index: usize,
    field_count: usize,
}

impl<'a> NastranIterator<'a> {
    fn new(iter: &'a mut Peekable<Iter<'a, u8>>) -> NastranIterator<'a> {
        return NastranIterator {
                   iter: iter,
                   is_comma: false,
                   is_double: false,
                   field_index: 0,
                   line_index: 0,
                   field_count: 0,
               };
    }

    fn next_char(&mut self) -> Option<&u8> {
        self.field_index += 1;
        self.line_index += 1;
        return self.iter.next();
    }

    fn reset_line(&mut self) {
        self.is_comma = false;
        self.is_double = false;
        self.field_count = 0;
    }

    fn parse_file(&mut self) -> Option<Deck> {
        let mut cards = vec![];
        while let Some(c) = self.parse_line() {
            cards.push(c);
        }
        return Some(Deck { cards: cards });
    }

    fn parse_line(&mut self) -> Option<Card> {
        self.reset_line();
        let mut fields = vec![];
        if let Some(first_field) = self.parse_first_field() {
            fields.push(first_field);
        } else {
            return None;
        }
        let v = self.iter
            .take_while(|&c| chars::is_newline(c))
            .cloned()
            .collect();
        return Some(Card {
                        fields: fields,
                        comment: Some(v),
                    });
    }

    fn parse_first_continuation(&mut self) -> Option<Field> {
        if let Some(&&c) = self.iter.peek() {
            if c == b'*' {
                self.is_double = true;
            }
        }
        let c = self.iter
            .take(8)
            .take_while(|&&c| c != b',' && c != b'\t')
            .cloned()
            .collect();
        return Some(Field::Continuation(c));
    }

    fn parse_first_string(&mut self) -> Option<Field> {
        let mut string_started = false;
        let mut string_ended = false;
        let mut svec = vec![];
        let mut field_ended = false;
        {
            let mut it = self.iter.by_ref().take(8);
            while let Some(&c) = it.next() {
                if c == b',' {
                    self.is_comma = true;
                    field_ended = true;
                    break;
                } else if c == b'\t' {
                    field_ended = true;
                    break;
                } else if !string_started {
                    if chars::is_alpha(&c) {
                        string_started = true;
                        svec.push(c);
                    } else if c != b' ' {
                        println!("Expected Space or alpha");
                        return None;
                    }
                } else if string_started && !string_ended {
                    if !chars::is_alphanumeric(&&c) {
                        string_ended = true;
                        if c == b'*' {
                            self.is_double = true;
                        }
                    } else {
                        svec.push(c);
                    }
                } else if string_ended {
                    if !self.is_double && c == b'*' {
                        self.is_double = true;
                    } else if c != b' ' {
                        println!("Expected Space '{}'", c as char);
                        return None;
                    }
                }
            }
        }
        if !field_ended {
            let mut it = self.iter.by_ref();
            if let Some(&&c) = it.peek() {
                if c == b',' {
                    self.is_comma = true;
                    it.next();
                }
            }
        }
        if !string_started {
            Some(Field::Blank)
        } else {
            Some(Field::String(svec))
        }
    }

    fn parse_first_field(&mut self) -> Option<Field> {
        let n = min(self.iter.len(), 8);
        if n == 0 {
            return None;
        }
        return match self.iter.peek() {
                   Some(&&b'+') | Some(&&b'*') => self.parse_first_continuation(),
                   Some(_) => self.parse_first_string(),
                   None => None,
               };
    }

    // fn parse_string<I: Iterator>(&mut self, iter: I) -> Option<Field> {
    //     return None;
    // }

    fn parse_comma_field(&mut self) -> Option<Field> {
        let mut field_started = false;
        while let Some(&c) = self.iter.next() {
            if !field_started && c != b' ' {
                field_started = true;
            }
        }
        None
    }

    fn parse_field(&mut self) -> Option<Field> {
        if self.is_comma {
            self.parse_comma_field()
        } else {
            None
        }
    }
}

mod chars {
    pub fn is_newline(&c: &u8) -> bool {
        c == b'\r' || c == b'\n'
    }
    pub fn is_field_end(&c: &u8) -> bool {
        c == b'\t' || c == b','
    }
    pub fn is_not_field_end(&c: &u8) -> bool {
        !is_field_end(&c)
    }
    pub fn is_alpha(&b: &u8) -> bool {
        (b >= b'a' && b <= b'z') || (b >= b'A' && b <= b'Z')
    }
    pub fn is_numeric(&b: &u8) -> bool {
        b >= b'0' && b <= b'9'
    }
    pub fn is_alphanumeric(&&b: &&u8) -> bool {
        is_alpha(&b) || is_numeric(&b)
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

fn read_first_field(line: &[u8]) -> (Field, CardFlags, &[u8]) {
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
    let first_field = (&line[..i_end]).to_owned();
    let remainder = &line[i_next..];
    return (Field::String(first_field), flags, remainder);
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
    return Ok(Field::String(field.to_owned()));
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
        return Ok(Field::String(field.to_owned())) 
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
    let (field, flags, remainder) = read_first_field(line);
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

