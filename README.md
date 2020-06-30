# nastran-rs

This is a rust crate for interacting with nastran files. It currently has basic datfile and op2 file parsers. And it has the beginnings of a python module for use from python

The op2 reader is written against NX Nastran 11 currently. 

## TODO

### datfile 
- Consider expanding support for individual card types with named fields
- Improve datfile test cases
- Add documentation

### op2 
- Improve error messages
- Add support for various data blocks in op2
- Add write capabilities for op2 files
- Add documentation

### python 
- Add full support for parsing a datfile and iterating through the results
- Add ability to output to pandas
- Add documentation

### util
- Add helper utilities for interacting with datfiles
