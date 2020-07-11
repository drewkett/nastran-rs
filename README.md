# nastran-rs

This is a rust crate for interacting with nastran files. It currently has basic
datfile and op2 file parsers. And it has the beginnings of a python module for
use from python

The op2 reader is written against NX Nastran 11 currently.

## TODO

### datfile

- v1 Parser
  - Needs continuation merging
  - Start implementing a few basic card types
  - I'm not confident comma line parsing is correct. It currently can get
    through a file without error, but I think there are edge cases that can be
    wrong. It probably needs another pass at it where the state of the parser is
    properly tracked with enums rather than just forcing everything through
    iterators
  - Needs Testing
  - Add better error messages. Should at least point to line that error occured
    on
  - Needs INCLUDE support
  - Needs a header parser
  - Add documentation
  - Should be split up across multiple files

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
