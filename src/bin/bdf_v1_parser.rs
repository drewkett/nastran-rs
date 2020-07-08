use bstr::ByteSlice;
use nastran::bdf::v1::parser::{parse_bytes, Result};
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
    let deck = parse_bytes(bytes)?;
    for card in deck {
        print!("original = {}", card.original.as_bstr());
        println!("result   = {:?}", card.data);
    }
    Ok(())
}
