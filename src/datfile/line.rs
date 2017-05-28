use super::{Card, Field};

use std::cmp::min;
use nom::{self, IResult, is_space, rest};

use super::field;

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


pub fn parse_line(line: &[u8]) -> IResult<&[u8], Card> {
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

named!(pub split_line<Card>,map!(
    tuple!(
        flat_map!(take_m_n_while!(0,80,call!(|c| c != b'$' && c != b'\n')),parse_line),
        alt!(
            map!(alt!(eof!()|tag!("\n")),|_| None) |
            map!(preceded!(opt!(tag!("$")),take_until_and_consume!("\n")),|c| Some(c))
        )
    )
,|(card,comment)| Card { comment, .. card}));
