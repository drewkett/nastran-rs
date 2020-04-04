use crate::datfile::Field;
use std::fmt;

#[derive(PartialEq, Clone)]
pub struct Card<'a> {
    pub first: Option<Field<'a>>,
    pub fields: Vec<Field<'a>>,
    pub continuation: &'a str,
    pub comment: Option<&'a [u8]>,
    pub is_double: bool,
    pub is_comma: bool,
    pub unparsed: Option<&'a [u8]>,
}

impl<'a> Card<'a> {
    pub fn merge(&mut self, card: Card<'a>) {
        // TODO Unsure if I should check for continuation match
        self.fields.extend_from_slice(card.fields.as_slice());
        self.is_double |= card.is_double;
        self.continuation = card.continuation;
        // TODO Not sure what to do with comments. Probably need a better data structure
    }
}

impl<'a> fmt::Debug for Card<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Card(")?;
        write!(f, "{:?},", self.first)?;
        for field in &self.fields {
            write!(f, "{:?},", field)?;
        }
        if self.continuation != "" {
            write!(f, "+{:?}", self.continuation)?;
        }
        if let Some(comment) = self.comment {
            write!(f, "Comment='{}',", String::from_utf8_lossy(comment))?;
        }
        if self.is_comma {
            write!(f, "comma,")?;
        }
        if self.is_double {
            write!(f, "double,")?;
        }
        if let Some(unparsed) = self.unparsed {
            write!(f, "Unparsed='{}',", String::from_utf8_lossy(unparsed))?;
        }
        write!(f, ")")
    }
}

impl<'a> fmt::Display for Card<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref first) = self.first {
            write!(f, "{}", first)?;
            let mut write_cont = false;
            if self.is_double {
                for field_chunk in self.fields.chunks(4) {
                    if write_cont {
                        write!(f, "+       \n*       ")?
                    }
                    for field in field_chunk {
                        write!(f, "{:16}", field)?
                    }
                    write_cont = true;
                }
            } else {
                for field_chunk in self.fields.chunks(8) {
                    if write_cont {
                        write!(f, "+       \n+       ")?
                    }
                    for field in field_chunk {
                        write!(f, "{}", field)?
                    }
                    write_cont = true;
                }
            }
            // TODO Need a better way to handle this. I complete card shouldn't have a continuation
            //try!(write!(f, "+{:7}", self.continuation));
        }
        if let Some(comment) = self.comment {
            if !comment.is_empty() && comment[0] != b'$' {
                write!(f, "$")?;
            }
            write!(f, "{}", String::from_utf8_lossy(comment))?;
        }
        write!(f, "")
    }
}
