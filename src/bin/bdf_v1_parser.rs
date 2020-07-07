use bstr::ByteSlice;
use std::fs::File;
use std::io;
use std::io::prelude::*;

pub fn main() -> io::Result<()> {
    let mut args = std::env::args();
    let _ = args
        .next()
        .ok_or(io::Error::new(io::ErrorKind::NotFound, "missing argument"))?;
    let filename = args
        .next()
        .ok_or(io::Error::new(io::ErrorKind::NotFound, "missing argument"))?;
    println!("{}", filename);
    let f = File::open(filename)?;
    let deck = nastran::bdf::v1::parser::parse_bytes(f.bytes())?;
    for card in deck {
        print!("original = {}", card.original.as_bstr());
        println!("result   = {}", card);
    }
    Ok(())
}
