use nastran::bdf::parser::parse_bytes;
use nastran::bdf::Result;

use std::io;

pub fn main() -> Result<()> {
    let mut args = std::env::args();
    let _ = args
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing argument"))?;
    let filename = args
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing argument"))?;
    println!("{}", filename);
    let bytes = std::fs::read(filename)?;
    for card in parse_bytes(&bytes)? {
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
