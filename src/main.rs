extern crate nastran;

pub fn main() {
    if let Some(d) = nastran::parse_buffer(b"PARAM,POST") {
        println!("{}",d);
    }
    if let Some(d) = nastran::parse_buffer(b"PARAM ,POST") {
        println!("{}",d);
    }
    if let Some(d) = nastran::parse_buffer(b"PARAM  ,POST") {
        println!("{}",d);
    }
    if let Some(d) = nastran::parse_buffer(b"PARAM   ,POST") {
        println!("{}",d);
    }
    if let Some(d) = nastran::parse_buffer(b"PARAM    ,POST") {
        println!("{}",d);
    }
    if let Some(d) = nastran::parse_buffer(b"PARAM,POST\nPARAM2,BLAH") {
        println!("{}",d);
    }
}
