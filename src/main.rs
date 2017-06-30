#![allow(dead_code)]
#![allow(unused_comparisons)]

#![cfg_attr(test,feature(test))]

#[cfg(test)]
extern crate test;

#[macro_use]
extern crate nom;
extern crate memmap;
extern crate ascii;
#[macro_use]
extern crate quick_error;
extern crate dtoa;
extern crate clap;
extern crate itertools;
extern crate num;
extern crate num_traits;

use std::fs::File;
use std::io::Write;

use memmap::{Mmap, Protection};
use clap::{Arg, App};

#[macro_use]
mod macros;
mod op2;
mod datfile;
mod errors;

pub fn main() {
    let matches = App::new("Nastran Reader")
        .arg(Arg::with_name("DATFILE").help(".dat file for reading").required(true).index(1))
        .arg(Arg::with_name("OUTPUT").help("output to file").short("o").takes_value(true))
        .arg(Arg::with_name("echo").long("echo").help("Output cards"))
        .get_matches();
    if let Some(filename) = matches.value_of("DATFILE") {
        let f = Mmap::open_path(filename, Protection::Read).unwrap();
        let sl = unsafe { f.as_slice() };
        let echo = matches.is_present("echo") || matches.is_present("OUTPUT");
        let deck = datfile::parse_buffer(sl).unwrap();
        if echo {
            if let Some(output_filename) = matches.value_of("OUTPUT") {
                if let Ok(mut f) = File::create(output_filename) {
                    for card in deck.cards {
                        write!(f,"{}\n",card).unwrap()
                    }
                } else {
                    println!("Couldn't open file '{}' for writing",output_filename)
                }
            } else {
                for card in deck.cards {
                    println!("{}",card)
                }
            }
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
