extern crate nastran;

#[macro_use]
extern crate nom;

extern crate memmap;

use std::fs::File;
use std::io::Read;

use memmap::{Mmap,Protection};

mod op2;

pub fn main() {
    let filename = "../../../Documents/op2/run.op2";
    let f = Mmap::open_path(filename, Protection::Read).unwrap();
    let sl = unsafe { f.as_slice() };

    let (_, data) = op2::read_op2(sl).unwrap();
    println!("{:?}", data.header);
    for block in data.blocks {
        println!("{:?}",block.header)
    }
}
