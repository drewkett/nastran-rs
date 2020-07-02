use super::super::{Deck, Result};

use std::collections::HashMap;

#[derive(Debug, PartialEq)]
struct DeckParser<'a> {
    buffer: &'a [u8],
    deck: Deck<'a>,
    continuations: HashMap<[u8; 8], usize>,
}

impl<'a> DeckParser<'a> {
    fn new(buffer: &'a [u8]) -> DeckParser<'a> {
        DeckParser {
            buffer,
            deck: Deck::new(),
            continuations: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    fn next_line(&mut self) -> &'a [u8] {
        if let Some(i) = self.buffer.iter().position(|&c| c == b'\n') {
            let (line, buffer) = self.buffer.split_at(i);
            self.buffer = buffer;
            line
        } else {
            let buffer = self.buffer;
            self.buffer = b"";
            buffer
        }
    }
}

pub fn parse_deck(buffer: &[u8]) -> Result<Deck> {
    let deck = DeckParser::new(buffer);
    Ok(deck.deck)
}
