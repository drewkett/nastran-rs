#![allow(dead_code)]
#![allow(unused_comparisons)]

#![cfg_attr(test,feature(test))]

#[cfg(test)] extern crate test;

#[macro_use] extern crate nom;
extern crate memmap;
extern crate ascii;
#[macro_use] extern crate error_chain;
extern crate dtoa;
extern crate clap;

use memmap::{Mmap, Protection};
use clap::{Arg, App};

#[macro_use] mod macros;
mod op2;
mod datfile;
mod errors;

pub fn main() {
    let matches = App::new("Nastran Reader")
        .arg(Arg::with_name("DATFILE")
                 .help(".dat file for reading")
                 .required(true)
                 .index(1))
        .get_matches();
    if let Some(filename) = matches.value_of("DATFILE") {
        let f = Mmap::open_path(filename, Protection::Read).unwrap();
        let sl = unsafe { f.as_slice() };
        let deck = datfile::parse_buffer(sl).unwrap();
        for card in deck.cards {
            println!("{}",card)
        }
    }
    // let f = Mmap::open_path(filename, Protection::Read).unwrap();
    // let sl = unsafe { f.as_slice() };

    // let (_, data) = op2::read_op2(sl).unwrap();
    // println!("{:?}", data.header);
    // for block in data.blocks {
    //     match block {
    //         op2::DataBlock::OUG(d) => {
    //             for (_, dataset) in d.record_pairs {
    //                 for data in dataset {
    //                     println!("{:?}", data.data);
    //                 }
    //             }
    //         }
    //         op2::DataBlock::GEOM1(b) => {
    //             for record in b.records {
    //                 println!("{:?}", record);
    //             }
    //         }
    //         op2::DataBlock::GEOM2(b) => {
    //             for record in b.records {
    //                 println!("{:?}", record);
    //             }
    //         }
    //         op2::DataBlock::GEOM4(b) => {
    //             for record in b.records {
    //                 println!("{:?}", record);
    //             }
    //         }
    //         op2::DataBlock::EPT(b) => {
    //             for record in b.records {
    //                 println!("{:?}", record);
    //             }
    //         }
    //         op2::DataBlock::DYNAMIC(b) => {
    //             for record in b.records {
    //                 println!("{:?}", record);
    //             }
    //         }
    //         op2::DataBlock::Generic(b) => {
    //             println!("{}", b.name);
    //         }
    //     }
    // }
}
