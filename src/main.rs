#![allow(dead_code)]

#[macro_use] extern crate nom;
extern crate memmap;
extern crate ascii;
#[macro_use] extern crate lazy_static;
extern crate regex;

use memmap::{Mmap, Protection};

mod op2;
mod datfile;

pub fn main() {
    let filename = "../../../Documents/op2/run.op2";
    let f = Mmap::open_path(filename, Protection::Read).unwrap();
    let sl = unsafe { f.as_slice() };

    let (_, data) = op2::read_op2(sl).unwrap();
    println!("{:?}", data.header);
    for block in data.blocks {
        match block {
            op2::DataBlock::OUG(d) => {
                for (_, dataset) in d.record_pairs {
                    for data in dataset {
                        println!("{:?}", data.data);
                    }
                }
            }
            op2::DataBlock::GEOM1(b) => {
                for record in b.records {
                    println!("{:?}", record);
                }
            }
            op2::DataBlock::GEOM2(b) => {
                for record in b.records {
                    println!("{:?}", record);
                }
            }
            op2::DataBlock::GEOM4(b) => {
                for record in b.records {
                    println!("{:?}", record);
                }
            }
            op2::DataBlock::EPT(b) => {
                for record in b.records {
                    println!("{:?}", record);
                }
            }
            op2::DataBlock::DYNAMIC(b) => {
                for record in b.records {
                    println!("{:?}", record);
                }
            }
            op2::DataBlock::Generic(b) => {
                println!("{}", b.name);
            }
        }
    }
}
