use std::convert::{TryFrom, TryInto};
use std::io;

use crate::bdf::{
    parser::{parse_bytes_iter, BulkCard},
    Error, Result,
};

#[derive(Debug)]
pub struct Grid {
    id: i32,
    cid: i32,
    x: f64,
    y: f64,
    z: f64,
    ocid: i32,
}

impl TryFrom<BulkCard> for Grid {
    type Error = Error;
    fn try_from(card: BulkCard) -> Result<Self> {
        match card.card_type().as_ref() {
            Some(b"GRID   ") => {}
            Some(c) => return Err(Error::UnexpectedCardType(*b"GRID   ", *c)),
            None => return Err(Error::UnexpectedCardType(*b"GRID   ", *b"       ")),
        }
        let mut iter = card.fields().iter().cloned();
        let id = iter.next().unwrap_or_default().try_into()?;
        let cid = iter.next().map(TryInto::try_into).transpose()?.unwrap_or(0);
        let x = iter
            .next()
            .map(TryInto::try_into)
            .transpose()?
            .unwrap_or(0.0);
        let y = iter
            .next()
            .map(TryInto::try_into)
            .transpose()?
            .unwrap_or(0.0);
        let z = iter
            .next()
            .map(TryInto::try_into)
            .transpose()?
            .unwrap_or(0.0);
        let ocid = iter.next().map(TryInto::try_into).transpose()?.unwrap_or(0);
        Ok(Grid {
            id,
            cid,
            x,
            y,
            z,
            ocid,
        })
    }
}

#[derive(Debug, Default)]
pub struct Deck {
    grids: Vec<Grid>,
}

impl Deck {
    pub fn from_bytes<I>(iter: I) -> Result<Self>
    where
        I: Iterator<Item = io::Result<u8>>,
    {
        let mut deck: Deck = Default::default();
        let mut iter = parse_bytes_iter(iter);
        while let Some(card) = iter.next() {
            let card = card?;
            match card.card_type().as_ref() {
                Some(b"GRID   ") => deck.grids.push(card.try_into()?),
                _ => {}
            }
        }
        Ok(deck)
    }
}
