
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
    count: usize
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

    pub fn is_newline(&c: &u8) -> bool {
        c == LF || c == CR
    }
}

fn parse_first_field(line: &[u8]) -> Option<Field> {
    let s = String::from_utf8_lossy(line).into_owned();
    Some(Field::String(s))
}

fn parse_line(line: &[u8]) -> Option<Card> {
    if line.len() == 0 {
        return None
    }
    let mut fields = vec![];
    if let Some(field) = parse_first_field(line) {
        fields.push(field)
    } else {
        return None
    }
    Some(Card {fields: fields})
}

pub fn parse_buffer(buffer: &[u8]) -> Deck {
    let mut cards = vec![];
    for line in buffer.split(chars::is_newline) {
        if line.len() == 0 {
            continue
        }
        if let Some(c) = parse_line(line) {
            cards.push(c)
        }
    }
    Deck { cards: cards }
}
