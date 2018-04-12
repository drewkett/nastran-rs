#![cfg_attr(test,feature(test))]

#[cfg(test)]
extern crate test;

#[macro_use]
extern crate quick_error;
extern crate dtoa;
extern crate nom;
extern crate itertools;
extern crate num;
extern crate num_traits;

pub mod datfile;
pub mod errors;
