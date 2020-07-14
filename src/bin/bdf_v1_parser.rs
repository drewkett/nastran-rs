use nastran::bdf::v1::parser::{parse_bytes_iter, Result};
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
    let mut iter = parse_bytes_iter(bytes);
    while let Some(card) = iter.next() {
        let card = card?;
        // if card.original().is_empty() {
        //     println!("original = ");
        // } else {
        //     print!("original = {}", card.original().as_bstr());
        // }
        // print!("result  = {}", card);
        print!("{}", card);
    }
    Ok(())
}
