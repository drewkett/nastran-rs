# TODO

## bdf

- Add more card types
- Consider whether traits as implemented are a good idea
- Consider exactly what the goals/non-goals with these structures

### parser

- FirstField interface is awkward. Could use better names too
- Make new line detectable or settable with a builder
- I'm not confident comma line parsing is correct. It currently can get through
  a file without error, but I think there are edge cases that can be wrong. It
  probably needs another pass at it where the state of the parser is properly
  tracked with enums rather than just forcing everything through iterators
- Needs Testing
- Add better error messages. Should at least point to line that error occured on
- Needs INCLUDE support
  - Make sure end data works better
- Needs a header parser
- Add documentation
- Should be split up across multiple files
- Verify card display code is working
- Implement macro for card automating conversion of BulkCard to Card
- Field maybe shouldn't be parsed immediately since some cards can be chars or
  int
- Consider making a wrapping type for id numbers to validate they are betweeen 1
  and 99999999
- Need new data structure for maintaining the card order for a deck, thats also
  COW

## OP2

- Improve error messages
- Add support for various data blocks in op2
- Add write capabilities for op2 files
- Add documentation

## python

- Add full support for parsing a bdf and iterating through the results
- Add ability to output to pandas
- Add documentation

## util

- Create a diff utility for bdfs
