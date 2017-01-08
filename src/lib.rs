
use std::cmp::min;
use std::iter::Peekable;
use std::slice::Iter;

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
    flags: CardFlags,
    comment: Option<String>,
}

#[derive(Debug,PartialEq)]
pub struct Deck {
    pub cards: Vec<Card>,
}

type CharIterator<'a> = Peekable<Iter<'a, u8>>;

mod chars {
    pub const LF: u8 = '\n' as u8;
    pub const CR: u8 = '\r' as u8;
    pub const TAB: u8 = '\t' as u8;
    pub const COMMA: u8 = ',' as u8;
    pub const SPACE: u8 = ' ' as u8;
    pub const STAR: u8 = '*' as u8;
    pub const PLUS: u8 = '+' as u8;

    pub fn is_newline(&c: &u8) -> bool {
        c == LF || c == CR
    }
    // pub fn is_field_end(&c: &u8) -> bool {
    //     c == TAB || c == COMMA
    // }
    // pub fn is_not_field_end(&c: &u8) -> bool {
    //     !is_field_end(&c)
    // }
    pub fn is_alpha(&b: &u8) -> bool {
        let c = b as char;
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')
    }
    pub fn is_numeric(&b: &u8) -> bool {
        let c = b as char;
        c >= '0' && c <= '9'
    }
    pub fn is_alphanumeric(&&b: &&u8) -> bool {
        is_alpha(&b) || is_numeric(&b)
    }
    pub fn is_space(&&b: &&u8) -> bool {
        b == SPACE
    }
    // pub fn is_not_tab(&&c: &&u8) -> bool {
    //     c != TAB
    // }
    // pub fn is_comma(&c: &u8) -> bool {
    //     c == COMMA
    // }
    // pub fn is_not_comma(&&c: &&u8) -> bool {
    //     c != COMMA
    // }
    // pub fn is_comma_tab(&c: &u8) -> bool {
    //     c == TAB || c == COMMA
    // }
    // pub fn is_not_comma_tab(&&c: &&u8) -> bool {
    //     c != TAB && c != COMMA
    // }
    // pub fn is_star_or_plus(&c: &u8) -> bool {
    //     c == STAR || c == PLUS
    // }
}

fn parse_first_continuation(line: &[u8], is_double: bool) -> Option<(Field, CardFlags, CharIterator)> {
    let len1 = min(8, line.len());
    // let len2 = min(10, line.len());
    for i in 0..len1 {
        println!("{}", i)
    }
    return None;
}

// fn take_spaces(line: &[u8]) -> usize {
//     return line.iter().take_while(chars::is_space).count();
// }

// fn take_string(line: &[u8]) -> usize {
//     if line.len() == 0 {
//         return 0;
//     }
//     if chars::is_alpha(&line[0]) {
//         return 1 + line.iter().take_while(chars::is_alphanumeric).count();
//     } else {
//         return 0;
//     }
// }

fn parse_first_string<'a>(line: &[u8]) -> Option<(Field, CardFlags, CharIterator)>
{
    let mut flags = CardFlags {
        is_double: false,
        is_comma: false,
    };
    // let mut remainder = line;
    // match line.iter().take(10).take_while(chars::is_not_tab).position(chars::is_comma) {
    //     Some(i) => {
    //         flags.is_comma = true;
    //         remainder = &line[i..];
    //     }
    //     None => (),
    // }
    // let mut field_len = min(8, line.len());
    let mut string_started = false;
    let mut string_ended = false;
    let mut svec = vec![];
    let mut it = line.iter().peekable();
    let mut field_ended = false;
    {
        let mut it8 = it.by_ref().take(8);
        while let Some(&c) = it8.next() {
            if c == chars::COMMA {
                flags.is_comma = true;
                field_ended = true;
                break;
            } else if c == chars::TAB {
                field_ended = true;
                break;
            } else if !string_started {
                if chars::is_alpha(&c) {
                    string_started = true;
                    svec.push(c);
                } else if !chars::is_space(&&c) {
                    println!("Expected Space or alpha");
                    return None;
                }
            } else if string_started && !string_ended {
                if !chars::is_alphanumeric(&&c) {
                    string_ended = true;
                    if c == chars::STAR {
                        flags.is_double = true;
                    }
                } else {
                    svec.push(c);
                }
            } else if string_ended {
                if !flags.is_double && c == chars::STAR {
                    flags.is_double = true;
                } else if c != chars::SPACE {
                    println!("Expected Space '{}'", c as char);
                    return None;
                }
            }
        }
    }
    if !field_ended {
        {
            let mut it2 = it.by_ref();
            if let Some(&&c) = it2.peek() {
                if c == chars::COMMA {
                    flags.is_comma = true;
                    it2.next();
                }
            }
        }
    }
    if !string_started {
        Some((Field::Blank, flags, it))
    } else {
        match String::from_utf8(svec) {
            Ok(s) => Some((Field::String(s), flags, it)),
            _ => None,
        }
    }
}

fn parse_first_field(line: &[u8]) -> Option<(Field, CardFlags, CharIterator)> {
    let n = min(line.len(), 8);
    if n == 0 {
        return None;
    }
    return match line[0] {
        chars::PLUS => parse_first_continuation(&line[1..], false),
        chars::STAR => parse_first_continuation(&line[1..], true),
        _ => parse_first_string(&line[0..]),
    };
}

fn parse_line(line: &[u8]) -> Option<Card> {
    if line.len() == 0 {
        return None;
    }
    let mut fields = vec![];
    if let Some((first_field, flags, _)) = parse_first_field(line) {
        fields.push(first_field);
        return Some(Card {
            fields: fields,
            flags: flags,
            comment: None,
        });
    } else {
        return None;
    }
}

// fn parse_string<'a, I>(mut it: I) -> Option<Field>
//     where I: Iterator<Item = &'a u8>
// {
//     let mut started = false;
//     let mut ended = false;
//     let mut string = vec![];
//     while let Some(b) = it.next() {
//         if !started {
//             if chars::is_space(&b) {
//                 continue;
//             } else if chars::is_alpha(b) {
//                 started = true;
//                 string.push(*b);
//             } else {
//                 return None;
//             }
//         } else if started && !ended {
//             if chars::is_alphanumeric(&b) {
//                 string.push(*b);
//             } else if chars::is_space(&b) {
//                 ended = true
//             } else {
//                 return None;
//             }
//         } else {
//             if chars::is_space(&b) {
//                 continue;
//             } else {
//                 return None;
//             }
//         }
//     }
//     if let Ok(s) = String::from_utf8(string) {
//         return Some(Field::String(s));
//     } else {
//         return None;
//     }
// }

pub fn parse_buffer(buffer: &[u8]) -> Option<Deck> {
    let mut cards = vec![];
    for mut line in buffer.split(chars::is_newline) {
        if line.len() == 0 {
            continue;
        }
        // let mut it = line.iter();
        if let Some(c) = parse_line(&mut line) {
            cards.push(c)
        }
    }
    Some(Deck { cards: cards })
}
