
mod field;

use std::fmt;
use std::str;
use std::cmp;
use std::collections::HashMap;
use std::ops::IndexMut;

use dtoa;

use errors::*;
pub use self::field::{maybe_field, maybe_any_field};


//TODO Need to make sure right number of fields are being output for card
#[derive(PartialEq, Clone)]
pub enum Field<'a> {
    Blank,
    Int(i32),
    Float(f32),
    Double(f64),
    Continuation(&'a str),
    DoubleContinuation(&'a str),
    String(&'a str),
    DoubleString(&'a str),
}

impl<'a> fmt::Debug for Field<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Field::Blank => write!(f, "Blank"),
            Field::Int(i) => write!(f, "Int({})", i),
            Field::Float(d) => write!(f, "Float({})", d),
            Field::Double(d) => write!(f, "Double({})", d),
            Field::Continuation(c) => write!(f, "Continuation('{}')", c),
            Field::String(s) => write!(f, "String('{}')", s),
            Field::DoubleContinuation(s) => write!(f, "DoubleContinuation('{}')", s),
            Field::DoubleString(s) => write!(f, "DoubleString('{}')", s),
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
            Field::Continuation(c) => write!(f, "+{:7}", c),
            Field::String(s) => write!(f, "{:8}", s),
            Field::DoubleContinuation(c) => write!(f, "*{:7}", c),
            Field::DoubleString(s) => write!(f, "{:7}*", s),
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct Card<'a> {
    pub first: Field<'a>,
    pub fields: Vec<Field<'a>>,
    pub continuation: &'a str,
    pub comment: Option<&'a [u8]>,
    pub is_double: bool,
    pub is_comma: bool,
    pub unparsed: Option<&'a [u8]>,
}

impl<'a> Card<'a> {
    fn merge(&mut self, card: Card<'a>) {
        // TODO Unsure if I should check for continuation match
        self.fields.extend_from_slice(card.fields.as_slice());
        self.is_double |= card.is_double;
        self.continuation = card.continuation;
        // TODO Not sure what to do with comments. Probably need a better data structure
    }
}

impl<'a> fmt::Debug for Card<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "Card("));
        try!(write!(f, "{:?},", self.first));
        for field in &self.fields {
            try!(write!(f, "{:?},", field));
        }
        if self.continuation != "" {
            try!(write!(f, "+{:?}", self.continuation));
        }
        if let Some(comment) = self.comment {
            try!(write!(
                f,
                "Comment='{}',",
                unsafe { str::from_utf8_unchecked(comment) }
            ));
        }
        if self.is_comma {
            try!(write!(f, "comma,"));
        }
        if self.is_double {
            try!(write!(f, "double,"));
        }
        if let Some(unparsed) = self.unparsed {
            try!(write!(f, "Unparsed='{}',", unsafe {
                str::from_utf8_unchecked(unparsed)
            }));
        }
        write!(f, ")")
    }
}


impl<'a> fmt::Display for Card<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.first != Field::Blank || !self.fields.is_empty() {
            try!(write!(f, "{}", self.first));
            for field in &self.fields {
                try!(write!(f, "{}", field));
            }
            try!(write!(f, "+{}", self.continuation));
        }
        if let Some(comment) = self.comment {
            if !comment.is_empty() && comment[0] != b'$' {
                try!(write!(f, "$"));
            }
            try!(write!(
                f,
                "{}",
                unsafe { str::from_utf8_unchecked(comment) }
            ));
        }
        write!(f, "")
    }
}

#[derive(Debug, PartialEq)]
pub struct Deck<'a> {
    pub cards: Vec<Card<'a>>,
    pub header: Option<&'a [u8]>,
    pub unparsed: Option<&'a [u8]>,
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
impl<'a> Deck<'a> {
    pub fn new() -> Deck<'a> {
        Deck {
            cards: vec![],
            header: None,
            unparsed: None,
        }
    }

    pub fn set_header(&mut self, header: &'a [u8]) {
        self.header = Some(header);
    }
    pub fn set_unparsed(&mut self, unparsed: &'a [u8]) {
        self.unparsed = Some(unparsed);
    }

    pub fn add_card(&mut self, card: Card<'a>) -> Result<()> {
        self.cards.push(card);
        Ok(())
    }
}

impl<'a> From<WorkingDeck<'a>> for Deck<'a> {
    fn from(working_deck: WorkingDeck<'a>) -> Self {
        working_deck.deck
    }
}

#[derive(Debug, PartialEq)]
pub struct WorkingDeck<'a> {
    deck: Deck<'a>,
    continuations: HashMap<&'a str, usize>,
}

impl<'a> WorkingDeck<'a> {
    pub fn new() -> WorkingDeck<'a> {
        WorkingDeck {
            deck: Deck {
                cards: vec![],
                header: None,
                unparsed: None,
            },
            continuations: HashMap::new(),
        }
    }

    pub fn add_card(&mut self, card: Card<'a>) -> Result<()> {
        match card.first {
            Field::Continuation(c) |
            Field::DoubleContinuation(c) => {
                let index = self.continuations.remove(c).ok_or(
                    ErrorKind::UnmatchedContinuation(
                        c.to_owned(),
                    ),
                )?;
                let mut existing = self.deck.cards.index_mut(index);
                self.continuations.insert(card.continuation, index);
                existing.merge(card);
            }
            Field::String(_) |
            Field::DoubleString(_) => {
                self.continuations.insert(card.continuation, self.deck.cards.len());
                self.deck.cards.push(card);
            }
            Field::Blank => {
                let index = self.continuations.remove("").ok_or(
                    ErrorKind::UnmatchedContinuation(
                        "".to_owned(),
                    ),
                )?;
                let mut existing = self.deck.cards.index_mut(index);
                self.continuations.insert(card.continuation, index);
                existing.merge(card);
            }
            _ => unreachable!(),
        };
        Ok(())
    }
    pub fn set_header(&mut self, header: &'a [u8]) {
        self.deck.set_header(header);
    }
    pub fn set_unparsed(&mut self, unparsed: &'a [u8]) {
        self.deck.set_unparsed(unparsed);
    }
}

fn float_to_8(f: f32) -> String {
    // FIXME: can be improved
    let mut buf = [b' '; 8];
    if let Ok(n) = dtoa::write(&mut buf[..], f) {
        unsafe { String::from_utf8_unchecked(buf[..n].to_vec()) }
    } else {
        format!("{:8e}", f)
    }
}

fn double_to_8(f: f64) -> String {
    // FIXME: can be improved
    let mut buf = [b' '; 8];
    if let Ok(n) = dtoa::write(&mut buf[..], f) {
        unsafe { String::from_utf8_unchecked(buf[..n].to_vec()) }
    } else {
        format!("{:8e}", f)
    }
}

fn split_line(buffer: &[u8]) -> (&[u8], &[u8]) {
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

struct FirstField<'a> {
    field: Field<'a>,
    is_comma: bool,
    remainder: &'a [u8],
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
    let field = field::maybe_first_field(field_buffer)?;
    Ok(FirstField {
        field,
        is_comma,
        remainder,
    })
}

fn get_short_slice(buffer: &[u8]) -> Option<(&[u8], &[u8])> {
    if buffer.is_empty() {
        return None;
    }
    let n = cmp::min(buffer.len(), 8);
    let sl = &buffer[..n];
    if let Some(j) = sl.iter().position(|&b| b == b'\t') {
        return Some((&buffer[..j], &buffer[j + 1..]));
    } else {
        return Some((sl, &buffer[n..]));
    }
}

fn get_long_slice(buffer: &[u8]) -> Option<(&[u8], &[u8])> {
    if buffer.is_empty() {
        return None;
    }
    let n = cmp::min(buffer.len(), 16);
    let sl = &buffer[..n];
    if let Some(j) = sl.iter().position(|&b| b == b'\t') {
        return Some((&buffer[..j], &buffer[j + 1..]));
    } else {
        return Some((sl, &buffer[n..]));
    }
}

pub fn parse_line(buffer: &[u8]) -> Result<Card> {
    let (content, comment) = split_line(buffer);
    let FirstField {
        field,
        is_comma,
        mut remainder,
    } = read_first_field(content)?;
    let is_double = match field {
        Field::DoubleContinuation(_) |
        Field::DoubleString(_) => true,
        _ => false,
    };
    let mut fields = vec![];
    let mut continuation = "";
    if is_comma {
        let mut i = 2;
        let mut field_count = 0;
        let mut check_cont = false;
        for sl in remainder.split(|&b| b == b',') {
            if field_count == 8 {
                let f = field::maybe_any_field(sl)?;
                match f {
                    Field::Continuation(_) |
                    Field::DoubleContinuation(_) => {
                        // TODO Need to handle continuations in comma separated
                        return Err(ErrorKind::UnexpectedContinuation.into());
                    }
                    _ => fields.push(f),

                }
                field_count = 1;
            } else {
                fields.push(field::maybe_field(sl)?);
                field_count += 1;
            }
            i += 1;
        }
        while field_count < 8 {
            fields.push(Field::Blank);
            field_count += 1;
        }
        remainder = b"";
    } else if is_double {
        for _ in 0..4 {
            if let Some(pair) = get_long_slice(remainder) {
                let sl = pair.0;
                remainder = pair.1;
                fields.push(field::maybe_field(sl)?)
            } else {
                fields.push(Field::Blank);
            }
        }
        if !remainder.is_empty() {
            if let Some(pair) = get_short_slice(remainder) {
                let sl = pair.0;
                remainder = pair.1;
                continuation = field::trailing_continuation(sl)?;
            }
        }
    } else {
        for _ in 0..8 {
            if let Some(pair) = get_short_slice(remainder) {
                let sl = pair.0;
                remainder = pair.1;
                fields.push(field::maybe_field(sl)?)
            } else {
                fields.push(Field::Blank);
            }
        }
        if !remainder.is_empty() {
            if let Some(pair) = get_short_slice(remainder) {
                let sl = pair.0;
                remainder = pair.1;
                continuation = field::trailing_continuation(sl)?;
            }
        }
    }

    Ok(Card {
        first: field,
        fields,
        continuation,
        comment: option_from_slice(comment),
        is_double,
        is_comma,
        unparsed: option_from_slice(remainder),
    })
}

pub fn read_comments(input_buffer: &[u8]) -> &[u8] {
    let mut buffer = input_buffer;
    while let Some(j) = buffer.iter().position(|&c| c == b'\n') {
        let mut line_iter = buffer.iter().take(j);
        let mut is_comment_or_blank = true;
        while let Some(&c) = line_iter.next() {
            match c {
                b' ' => (),
                b'$' => break,
                _ => {
                    is_comment_or_blank = false;
                    break;
                }
            }
        }
        if !is_comment_or_blank {
            return buffer;
        } else {
            buffer = &buffer[j + 1..]
        }
    }
    return buffer;
}

pub fn read_header(input_buffer: &[u8]) -> (Option<&[u8]>, &[u8]) {
    let mut header = None;
    let mut buffer = read_comments(input_buffer);
    if !buffer.is_empty() {
        let is_header = match buffer[0] {
            b'I' => buffer.len() > 4 && &buffer[..4] == b"INIT",
            b'N' => buffer.len() > 7 && &buffer[..7] == b"NASTRAN",
            _ => (false),
        };
        if is_header {
            let mut header_end = None;
            // Loop through lines, looking for BEGIN [BULK]
            while let Some(j) = buffer.iter().position(|&c| c == b'\n') {
                let line = &buffer[..j];
                if !line.is_empty() {
                    if line.len() > 5 && &line[..5] == b"BEGIN" {
                        header_end = Some(input_buffer.len() - buffer.len());
                        buffer = &buffer[j + 1..];
                        break;
                    }
                }
                buffer = &buffer[j + 1..]
            }
            if let Some(j) = header_end {
                header = Some(&input_buffer[..j]);
            } else {
                header = Some(input_buffer);
                buffer = b"";
            }
        } else {
            // If no header is found. Comments are treated as cards and not as a part of the header
            buffer = input_buffer;
        }
    }
    (header, buffer)
}


pub fn parse_buffer(input_buffer: &[u8]) -> Result<Deck> {
    let mut deck = WorkingDeck::new();
    let mut line_num = 1;
    let (header, mut buffer) = read_header(input_buffer);
    if let Some(h) = header {
        deck.set_header(h);
    }
    loop {
        if buffer.is_empty() {
            break;
        }
        if let Some(j) = buffer.iter().position(|&c| c == b'\n') {
            let mut line = &buffer[..j];
            if j > 0 && line[j - 1] == b'\r' {
                line = &buffer[..j - 1]
            }
            let card = parse_line(line).chain_err(|| {
                format!("Error parsing line {}", line_num)
            })?;
            // Check for ENDDATA. If found, drop the card and set remaining buffer to unparsed
            if card.first == Field::String("ENDDATA") {
                deck.set_unparsed(&buffer[j + 1..]);
                break;
            } else {
                deck.add_card(card)?;
                buffer = &buffer[j + 1..];
            }
        } else {
            let card = parse_line(buffer).chain_err(|| {
                format!("Error parsing line {}", line_num)
            })?;
            deck.add_card(card)?;
            break;
        }
        line_num += 1;
    }
    Ok(deck.into())
}
