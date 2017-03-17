extern crate nastran;

#[macro_use]
extern crate nom;

use std::fs::File;
use std::io::Read;

mod op2;

pub fn main() {
    let mut f = File::open("../../../Documents/op2/run.op2").unwrap();
    let mut b = vec![];
    f.read_to_end(&mut b);

    let (_,data) = op2::read_op2(b.as_slice()).unwrap();
    println!("{:?}",data.header);

    // if let Some(d) = nastran::parse_buffer(b"PARAM,POST") {
    //     println!("{}",d);
    // }
    // if let Some(d) = nastran::parse_buffer(b"PARAM ,POST") {
    //     println!("{}",d);
    // }
    // if let Some(d) = nastran::parse_buffer(b"PARAM  ,POST") {
    //     println!("{}",d);
    // }
    // if let Some(d) = nastran::parse_buffer(b"PARAM   ,POST") {
    //     println!("{}",d);
    // }
    // if let Some(d) = nastran::parse_buffer(b"PARAM    ,POST") {
    //     println!("{}",d);
    // }
    // if let Some(d) = nastran::parse_buffer(b"PARAM,POST\nPARAM2,BLAH") {
    //     println!("{}",d);
    // }
}
