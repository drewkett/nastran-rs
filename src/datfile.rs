use std::cmp::min;
use std::iter::Peekable;
use std::slice::Iter;
use std::fmt;

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
    pub comment: Option<String>,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Card(");
        for field in self.fields.iter() {
            write!(f, "{:?},", field);
        }
        if let Some(ref c) = self.comment {
            write!(f, "Comment='{}'", c);
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
        write!(f, "Deck(\n");
        for card in self.cards.iter() {
            write!(f, "  {},\n", card);
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
            .take_while(|&&c| c != b'\r')
            .cloned()
            .collect();
        let comment = String::from_utf8(v).ok();
        return Some(Card {
                        fields: fields,
                        comment: comment,
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
        return match String::from_utf8(c) {
                   Ok(s) => Some(Field::Continuation(s)),
                   _ => None,
               };
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
            {
                let mut it = self.iter.by_ref();
                if let Some(&&c) = it.peek() {
                    if c == b',' {
                        self.is_comma = true;
                        it.next();
                    }
                }
            }
        }
        if !string_started {
            Some(Field::Blank)
        } else {
            match String::from_utf8(svec) {
                Ok(s) => Some(Field::String(s)),
                _ => None,
            }
        }
    }

    fn parse_first_field(&mut self) -> Option<Field> {
        let n = min(self.iter.len(), 8);
        if n == 0 {
            return None;
        }
        return match self.iter.peek() {
                   Some(&&b'+') |
                   Some(&&b'*') => self.parse_first_continuation(),
                   Some(_) => self.parse_first_string(),
                   None => None,
               };
    }

    fn parse_string<I: Iterator>(&mut self, iter: I) -> Option<Field> {
        return None;
    }

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
    let mut it = buffer.iter().peekable();
    let mut it = NastranIterator::new(&mut it);
    return it.parse_file();
}

