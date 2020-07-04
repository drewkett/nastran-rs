use std::fs::File;
use std::io::{self, Write};

use memmap::Mmap;

pub fn main() {
    let mut args = std::env::args();
    let filename = args.next().unwrap();
    let f = File::open(filename).unwrap();
    let mm = unsafe { Mmap::map(&f).unwrap() };
    let sl = mm.as_ref();
    let deck = nastran::bdf::v0::parse_buffer(sl).unwrap();
    if let Some(header) = deck.header {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(header).unwrap();
    }
    for card in deck.cards {
        println!("{}", card)
    }
}
