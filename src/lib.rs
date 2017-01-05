
#[derive(Debug,PartialEq)]
pub enum Field {
    Blank,
    Int(i32),
    Float(f32),
    Double(f64),
    Continuation(String),
    String(String),
}

struct CountedField {
    field: Field,
    count: usize,
}

#[derive(Debug,PartialEq)]
pub struct Card {
    pub fields: Vec<Field>,
}

#[derive(Debug,PartialEq)]
pub struct Deck {
    pub cards: Vec<Card>,
}

mod chars {
    pub const LF: u8 = '\n' as u8;
    pub const CR: u8 = '\r' as u8;
    pub const TAB: u8 = '\t' as u8;
    pub const COMMA: u8 = ',' as u8;
    pub const SPACE: u8 = ',' as u8;

    pub fn is_newline(&c: &u8) -> bool {
        c == LF || c == CR
    }
    pub fn is_field_end(&c: &u8) -> bool {
        c == TAB || c == COMMA
    }
    pub fn is_not_field_end(&c: &u8) -> bool {
        !is_field_end(&c)
    }
    pub fn is_alpha(&b: &u8) -> bool {
        let c = b as char;
        (c >= 'a' && c <= 'z') || (c >= 'A' && c<= 'Z')
    }
    pub fn is_numeric(&b: &u8) -> bool {
        let c = b as char;
        c >= '0' && c <= '9'
    }
    pub fn is_alphanumeric(&b: &u8) -> bool {
        is_alpha(&b) || is_numeric(&b)
    }
    pub fn is_space(&b: &u8) -> bool {
        b == SPACE
    }
}

fn parse_first_field(line: &[u8]) -> Option<Field> {
    let s = String::from_utf8_lossy(line).into_owned();
    Some(Field::String(s))
}

fn parse_line(line: &[u8]) -> Option<Card> {
    if line.len() == 0 {
        return None;
    }
    let mut fields = vec![];
    if let Some(field) = parse_first_field(line) {
        fields.push(field)
    } else {
        return None;
    }
    Some(Card { fields: fields })
}

fn parse_string<'a, I>(mut it: I)-> Option<Field>
    where I: Iterator<Item=&'a u8>
{
    let mut started = false;
    let mut ended = false;
    let mut string = vec![];
    while let Some(b) = it.next() {
        if !started {
            if chars::is_space(b) {
                continue
            } else if chars::is_alpha(b) {
                started = true;
                string.push(*b);
            } else {
                return None
            }
        } else if started && !ended {
            if chars::is_alphanumeric(b) {
                string.push(*b);
            } else if chars::is_space(b) {
                ended = true
            } else {
                return None;
            }
        } else  {
            if chars::is_space(b) {
                continue
            } else {
                return None;
            }
        }
    }
    if let Ok(s) = String::from_utf8(string) {
        return Some(Field::String(s))
    } else {
        return None
    }
}

pub fn parse_buffer(buffer: &[u8]) -> Option<Deck> {
    let mut cards = vec![];
    for line in buffer.split(chars::is_newline) {
        if line.len() == 0 {
            continue;
        }
        let mut it = line.iter();

        let mut fields = vec![];
        if let Some(f) = parse_string(it.by_ref().take(8).take_while(|b| chars::is_not_field_end(b))) {
            fields.push(f);
        } else {
            return None
        }
        if let Some(f) = parse_string(it.by_ref().take(8).take_while(|b| chars::is_not_field_end(b))) {
            fields.push(f);
        } else {
            return None
        }
        cards.push(Card{fields:fields})
    }
    Some(Deck { cards: cards })
}

