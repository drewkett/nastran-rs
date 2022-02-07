# nastran-rs

This is an experimental Rust crate for interacting with NASTRAN files. It currently has basic
datfile and op2 file parsers and a dummy python module for future use from python.

The first implementation of the the bdf reader is currently using `nom`, but I think a custom parser might work better for 
NASTRAN files.

The op2 reader is written against NX Nastran 11 currently.
