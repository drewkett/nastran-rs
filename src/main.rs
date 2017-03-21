extern crate nastran;

#[macro_use]
extern crate nom;

extern crate memmap;
use std::mem::{size_of, transmute};

use std::fs::File;
use std::io::Read;

use memmap::{Mmap, Protection};

mod op2;
mod ident;
mod oug;
mod keyed;
mod geom1;
mod geom2;
mod geom4;
mod ept;
mod dynamic;
mod generic;

pub fn main() {
    let filename = "../../../Documents/op2/run.op2";
    let f = Mmap::open_path(filename, Protection::Read).unwrap();
    let sl = unsafe { f.as_slice() };

    let (_, data) = op2::read_op2(sl).unwrap();
    println!("{:?}", data.header);
    for block in data.blocks {
        match block {
            op2::DataBlock::OUG(d) => {
                for (ident, dataset) in d.record_pairs {
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
