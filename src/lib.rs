#![cfg_attr(test,feature(test))]

#[cfg(test)]
extern crate test;

#[macro_use]
extern crate error_chain;
extern crate dtoa;
extern crate nom;

pub mod datfile;
pub mod errors;
