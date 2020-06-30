# nastran-rs

This is a rust crate for interacting with nastran files. It currently has basic datfile and op2 file parsers. And it has the beginnings of a python module for use from python

The op2 reader is written against NX Nastran 11 currently. 

## TODO

### datfile 
- Reorganize the parser so that there is a struct that owns the buffer and drives it
  - The goal with the parser would be to store references to every byte in the original 
    datfile so that it can be written back out byte for byte if needed
  - Do more to check for the safety of the parser. The fuzz checks are in place to help
    with that but I'm not sure how long they need to run to be comprehensive
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
