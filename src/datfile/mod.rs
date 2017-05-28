
use std::cmp::min;
use std::fmt;
use std::str;

use dtoa;
use nom;
use nom::{Slice, IResult, InputIter, is_space, rest};

use errors::{Result, ErrorKind};

mod field;

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
    let (_, mut card) = try_parse!(line, field::first_field);
    card.is_comma = is_comma;
    IResult::Done(remainder, card)
}

fn option_from_slice(sl: &[u8]) -> Option<&[u8]> {
    if !sl.is_empty() { Some(sl) } else { None }
}

named!(split_short_with_cont<(Vec<Field>,Option<&[u8]>)>, do_parse!(
    fields: many_m_n!(8,8,field::field_8) >>
    last_field: opt!(field::field_8_cont) >>
    take_while!(is_space) >>
    unparsed: map!(rest,option_from_slice) >>
    ({
        let mut mfields = fields;
        if let Some(f) = last_field { mfields.push(f) } ;
        (mfields, unparsed)
    })
));
named!(split_short_partial<(Vec<Field>,Option<&[u8]>)>, do_parse!(
    fields: many_m_n!(0,7,field::field_8) >>
    (fields, None)
));
named!(split_short<(Vec<Field>,Option<&[u8]>)>,alt_complete!(
    split_short_with_cont|split_short_partial
    ));

named!(split_long_with_cont<(Vec<Field>,Option<&[u8]>)>, do_parse!(
    fields: many_m_n!(4,4,field::field_16) >>
    last_field: opt!(field::field_8_cont) >>
    take_while!(is_space) >>
    unparsed: map!(rest,option_from_slice) >>
    ({
        let mut mfields = fields;
        if let Some(f) = last_field { mfields.push(f) } ;
        (mfields, unparsed)
    })
));

named!(split_long_partial<(Vec<Field>,Option<&[u8]>)>, do_parse!(
    fields: many_m_n!(0,3,field::field_16) >>
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
                let (_, field) = try_parse!(sl, field::short_field_cont);
                card.fields.push(field);
            } else {
                let (_, field) = try_parse!(sl, field::short_field);
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
