use std::fs::File;

use memmap::Mmap;

pub fn main() {
    let mut args = std::env::args();
    let _ = args.next().unwrap();
    let filename = args.next().unwrap();
    println!("{}", filename);
    let f = File::open(filename).unwrap();
    let mm = unsafe { Mmap::map(&f).unwrap() };
    let sl = mm.as_ref();
    let _deck = nastran::bdf::v1::parser::parse_deck(sl).unwrap();
}
