use super::super::{Deck, Result};

use std::collections::HashMap;

#[derive(Debug, PartialEq)]
struct LinesIterator<'a> {
    buffer: &'a [u8],
}

impl<'a> LinesIterator<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        Self { buffer }
    }

    #[allow(dead_code)]
    fn take(&mut self, i: usize) -> &'a [u8] {
        let (line, buffer) = self.buffer.split_at(i);
        self.buffer = buffer;
        return line;
    }

    #[allow(dead_code)]
    fn read_line_contents(&mut self) -> &'a [u8] {
        // j is the position in the line as nastran sees it, which includes
        // tab expansion
        let mut j = 0;
        for (i, c) in self.buffer.iter().enumerate() {
            if j == 80 || *c == b'$' || *c == b'\r' || *c == b'\n' {
                return self.take(i);
            } else if *c == b'\t' {
                j += 8 - (j % 8);
            } else {
                j += 1;
            }
        }
        return self.take(self.buffer.len());
    }

    #[allow(dead_code)]
    fn read_line_trailing(&mut self) -> &'a [u8] {
        let mut iter = self.buffer.iter().enumerate();
        while let Some((i, c)) = iter.next() {
            if *c == b'\n' {
                return self.take(i + 1);
            } else if *c == b'\r' {
                return match iter.next() {
                    Some((i, b'\n')) => self.take(i + 1),
                    _ => self.take(i + 1),
                };
            }
        }
        return self.take(self.buffer.len());
    }
}

impl<'a> Iterator for &mut LinesIterator<'a> {
    type Item = (&'a [u8], &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            return None;
        }
        let line_contents = self.read_line_contents();
        let line_trailing = self.read_line_trailing();
        if line_contents.is_empty() && line_trailing.is_empty() {
            None
        } else {
            Some((line_contents, line_trailing))
        }
    }
}

#[derive(Debug, PartialEq)]
struct DeckParser<'a> {
    lines: LinesIterator<'a>,
    deck: Deck<'a>,
    continuations: HashMap<[u8; 8], usize>,
}

impl<'a> DeckParser<'a> {
    fn new(buffer: &'a [u8]) -> DeckParser<'a> {
        DeckParser {
            lines: LinesIterator::new(buffer),
            deck: Deck::new(),
            continuations: HashMap::new(),
        }
    }
}

pub fn parse_deck(buffer: &[u8]) -> Result<Deck> {
    use bstr::ByteSlice;
    let mut deck = DeckParser::new(buffer);
    while let Some((line, trailing)) = (&mut deck.lines).next() {
        println!("{} :: {:?}", line.as_bstr(), trailing.as_bstr())
    }
    Ok(deck.deck)
}
