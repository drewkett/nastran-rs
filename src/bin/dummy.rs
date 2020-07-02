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
    // let f = amap::open_path(filename, Protection::Read).unwrap();
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
