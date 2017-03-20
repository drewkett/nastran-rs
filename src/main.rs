extern crate nastran;

#[macro_use]
extern crate nom;

extern crate memmap;
use std::mem::{size_of, transmute};

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
        match block {
            op2::DataBlock::OUG(d) => {
                for (ident,dataset) in d.record_pairs {
                    for data in dataset {
                        let a : &[f32;14] = unsafe { transmute(data) };
                        println!("{:?}",&a[..]);
                    }
                }
            },
            _ => {}
        }
        // if (block.trailer.name == "OUGV1   ") {
        // println!("{:?}",block.first_record);
        //     for record in block.records {
        //         println!("{:?}",record)
        //     }
        // }
    }
}
