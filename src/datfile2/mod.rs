
mod field;

use std::fmt;
use std::str;
use std::cmp;

use dtoa;

use errors::*;

#[derive(PartialEq)]
pub enum Field<'a> {
    Blank,
    Int(i32),
    Float(f32),
    Double(f64),
    Continuation(&'a [u8]),
    DoubleContinuation(&'a [u8]),
    String(&'a [u8]),
    DoubleString(&'a [u8]),
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
            Field::DoubleContinuation(s) => {
                write!(f,
                       "DoubleContinuation('{}')",
                       unsafe { str::from_utf8_unchecked(s) })
            }
            Field::DoubleString(s) => {
                write!(f,
                       "DoubleString('{}')",
                       unsafe { str::from_utf8_unchecked(s) })
            }
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
            Field::DoubleContinuation(c) => {
                write!(f, "*{:7}", unsafe { str::from_utf8_unchecked(c) })
            }
            Field::DoubleString(s) => write!(f, "{}*", unsafe { str::from_utf8_unchecked(s) }),
        }
    }
}

#[derive(PartialEq)]
pub struct Card<'a> {
    pub first: Field<'a>,
    pub fields: Vec<Field<'a>>,
    pub continuation: Option<Field<'a>>,
    pub comment: Option<&'a [u8]>,
    pub is_double: bool,
    pub is_comma: bool,
    pub unparsed: Option<&'a [u8]>,
}

impl<'a> fmt::Debug for Card<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "Card("));
        try!(write!(f, "{:?},", self.first));
        for field in &self.fields {
            try!(write!(f, "{:?},", field));
        }
        if let Some(ref field) = self.continuation {
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
        if self.first != Field::Blank || self.fields.len() > 0 {
            try!(write!(f, "{}", self.first));
            for field in &self.fields {
                try!(write!(f, "{}", field));
            }
            if let Some(ref field) = self.continuation {
                try!(write!(f, "{}", field));
            }
        }
        if let Some(comment) = self.comment {
            if comment.len() > 0 && comment[0] != b'$' {
                try!(write!(f, "$"));
            }
            try!(write!(f, "{}", unsafe { str::from_utf8_unchecked(comment) }));
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

fn split_line(buffer: &[u8]) -> (&[u8],&[u8]) {
    let mut i_comment = cmp::min(80, buffer.len());
    if let Some(i) = buffer.iter().position(|&c| c == b'$') {
        if i < i_comment {
            i_comment = i
        }
    }
    buffer.split_at(i_comment)
}

fn option_from_slice(sl: &[u8]) -> Option<&[u8]> {
    if !sl.is_empty() { Some(sl) } else { None }
}

struct FirstField <'a> {
    field: Field<'a> ,
    is_comma: bool,
    remainder: &'a [u8]
}

fn read_first_field(line: &[u8]) -> Result<FirstField> {
    let mut is_comma = false;
    let mut consume_next = false;
    let length = line.len();
    let size = cmp::min(length, 8);
    let mut i_end = size;
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
    let (field_buffer, mut remainder) = line.split_at(i_end);
    if consume_next {
        remainder = &remainder[1..];
    }
    let field = field::maybe_field(field_buffer)?;
    Ok(FirstField{
        field,
        is_comma,
        remainder
    })
}

pub fn parse_line(buffer: &[u8]) -> Result<Card> {
    let (content, comment) = split_line(buffer);
    let FirstField { field, is_comma, mut remainder } = read_first_field(content)?;
    let is_double = match field {
        Field::DoubleContinuation(_) | Field::DoubleString(_) => true,
        _ => false
    };
    let mut fields = vec![];
    if is_comma {
        let mut i = 2;
        for sl in remainder.split(|&b| b == b',') {
            fields.push(field::maybe_field(sl)?)
        }
        remainder = b"";
    }
    
    Ok(Card {
        first: field,
        fields: fields,
        continuation: None,
        comment: option_from_slice(comment),
        is_double,
        is_comma,
        unparsed: Some(remainder),
    })
}


pub fn parse_buffer(input_buffer: &[u8]) -> Result<Deck> {
    let mut line_num = 1;
    let mut cards = vec![];
    let mut buffer = input_buffer;
    loop {
        if buffer.len() == 0 {
            break;
        }
        if let Some(j) = buffer.iter().position(|&c| c == b'\n') {
            let mut line = &buffer[..j];
            if j > 0 && line[j - 1] == b'\r' {
                line = &buffer[..j - 1]
            }
            let card = parse_line(line).chain_err(|| format!("Error parsing line {}",line_num))?;
            cards.push(card);
            buffer = &buffer[j + 1..];
        } else {
            let card = parse_line(buffer).chain_err(|| format!("Error parsing line {}",line_num))?;
            cards.push(card);
            break;
        }
        line_num += 1;
    }
    Ok(Deck { cards })
}
