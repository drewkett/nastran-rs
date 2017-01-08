extern crate nastran;

pub fn main() {
    println!("{:?}",nastran::parse_buffer(b"PARAM,POST"));
    println!("{:?}",nastran::parse_buffer(b"PARAM ,POST"));
    println!("{:?}",nastran::parse_buffer(b"PARAM  ,POST"));
    println!("{:?}",nastran::parse_buffer(b"PARAM   ,POST"));
    println!("{:?}",nastran::parse_buffer(b"PARAM    ,POST")); 
}
