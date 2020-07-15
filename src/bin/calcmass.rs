use nastran::bdf::{deck::Deck, Result};
use std::fs::File;
use std::io;
use std::io::prelude::*;

pub fn main() -> Result<()> {
    let mut args = std::env::args();
    let _ = args
        .next()
        .ok_or(io::Error::new(io::ErrorKind::NotFound, "missing argument"))?;
    let filename = args
        .next()
        .ok_or(io::Error::new(io::ErrorKind::NotFound, "missing argument"))?;
    println!("{}", filename);
    let f = File::open(filename)?;
    let bytes = std::io::BufReader::new(f).bytes();

    let deck = Deck::from_bytes(bytes)?;
    // println!("{:?}", deck);
    Ok(())
}
