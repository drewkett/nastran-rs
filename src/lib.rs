#![allow(dead_code)]
#![allow(unused_comparisons)]

#![cfg_attr(test,feature(test))]
#[cfg(test)] extern crate test;

#[macro_use] extern crate nom;
extern crate memmap;
extern crate ascii;
#[macro_use] extern crate error_chain;
extern crate dtoa;

#[macro_use] mod macros;
pub mod op2;
pub mod datfile;
pub mod errors;
pub mod datfile2;
