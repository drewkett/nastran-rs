#![allow(dead_code)]

#[macro_use] extern crate nom;
extern crate memmap;
extern crate ascii;
#[macro_use] extern crate error_chain;

pub mod op2;
pub mod datfile;
pub mod errors;