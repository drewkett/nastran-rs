#![allow(dead_code)]
#![feature(test)]

extern crate test;

#[macro_use] extern crate nom;
extern crate memmap;
extern crate ascii;
extern crate regex;
#[macro_use] extern crate lazy_static;

pub mod op2;
pub mod datfile;