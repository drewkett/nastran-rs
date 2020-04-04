#![allow(dead_code)]
#![allow(unused_comparisons)]

use std::fs::File;
use std::io::{self, Write};

use clap::{App, Arg};
use memmap::{Mmap, Protection};

pub fn main() {
    let matches = App::new("Nastran Reader")
        .arg(
            Arg::with_name("DATFILE")
                .help(".dat file for reading")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .help("output to file")
                .short("o")
                .takes_value(true),
        )
        .arg(Arg::with_name("echo").long("echo").help("Output cards"))
        .get_matches();
    if let Some(filename) = matches.value_of("DATFILE") {
        let f = Mmap::open_path(filename, Protection::Read).unwrap();
        let sl = unsafe { f.as_slice() };
        let echo = matches.is_present("echo") || matches.is_present("OUTPUT");
        let deck = nastran::datfile::parse_buffer(sl).unwrap();
        if echo {
            if let Some(output_filename) = matches.value_of("OUTPUT") {
                if let Ok(mut f) = File::create(output_filename) {
                    if let Some(header) = deck.header {
                        f.write_all(header).unwrap();
                    }
                    for card in deck.cards {
                        write!(f, "{}\n", card).unwrap();
                    }
                } else {
                    println!("Couldn't open file '{}' for writing", output_filename)
                }
            } else {
                if let Some(header) = deck.header {
                    let stdout = io::stdout();
                    let mut handle = stdout.lock();
                    handle.write_all(header).unwrap();
                }
                for card in deck.cards {
                    println!("{}", card)
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
