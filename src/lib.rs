#![allow(dead_code)]

#![cfg_attr(test,feature(test))]
#[cfg(test)]
extern crate test;

#[macro_use]
extern crate nom;
extern crate memmap;
extern crate ascii;
#[macro_use]
extern crate error_chain;
extern crate dtoa;

pub mod op2;
pub mod datfile;
pub mod errors;
