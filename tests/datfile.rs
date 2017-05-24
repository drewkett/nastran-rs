extern crate nastran;
use nastran::datfile;

const DATFILE: &'static [u8] = b"\
PARAM,POST , 1 $ABC
PARAM, WTMASS,0.00259
+,1,2
+a,1,2


ABCDEF,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456
BLAH    123      1.+5   1e2     ABC
";

#[test]
fn comma_separated() {
    let res = match datfile::parse_buffer(DATFILE) {
        Ok(d) => d,
        Err(e) => {println!("{:?}",e); assert!(false); return}
    };
    assert_eq!(datfile::Deck {
                   cards: vec![datfile::Card {
                                   fields: vec![datfile::Field::String("PARAM".to_owned()),
                                                datfile::Field::String("POST".to_owned()),
                                                datfile::Field::Int(1)],
                                   comment: Some(b"$ABC".to_vec()),
                               },
                               datfile::Card {
                                   fields: vec![datfile::Field::String("PARAM".to_owned()),
                                                datfile::Field::String("WTMASS".to_owned()),
                                                datfile::Field::Float(0.00259)],
                                   comment: Some(b"".to_vec()),
                               },
                               datfile::Card {
                                   fields: vec![datfile::Field::Continuation("".to_owned()),
                                                datfile::Field::Int(1),
                                                datfile::Field::Int(2)],
                                   comment: Some(b"".to_vec()),
                               },
                               datfile::Card {
                                   fields: vec![datfile::Field::Continuation("a".to_owned()),
                                                datfile::Field::Int(1),
                                                datfile::Field::Int(2)],
                                   comment: Some(b"".to_vec()),
                               },
                               datfile::Card {
                                   fields: vec![datfile::Field::String("ABCDEF".to_owned()),
                                                datfile::Field::Int(123456),
                                                datfile::Field::Int(123456),
                                                datfile::Field::Int(123456),
                                                datfile::Field::Int(123456),
                                                datfile::Field::Int(123456),
                                                datfile::Field::Int(123456),
                                                datfile::Field::Int(123456),
                                                datfile::Field::Int(123456),
                                                datfile::Field::Int(123456),
                                                datfile::Field::Int(123456),
                                                datfile::Field::Int(123)],
                                   comment: Some(b"456,123456".to_vec()),
                               },
                               datfile::Card {
                                   fields: vec![datfile::Field::String("BLAH".to_owned()),
                                                datfile::Field::Int(123),
                                                datfile::Field::Float(1e5),
                                                datfile::Field::Float(1e2),
                                                datfile::Field::String("ABC".to_owned())],
                                   comment: Some(b"".to_vec()),
                               }
                               ],
               },
               res)
}

#[test]
fn test_parse() {
    assert_eq!(datfile::Field::Float(1.23),datfile::parse_field(b" 1.23 ").unwrap_or(datfile::Field::Blank));
    assert_eq!(datfile::Field::Float(1.24),datfile::parse_field(b" 1.24").unwrap_or(datfile::Field::Blank));
    assert_eq!(datfile::Field::Float(1.25),datfile::parse_field(b"1.25").unwrap_or(datfile::Field::Blank));
    assert_eq!(datfile::Field::Float(1.26),datfile::parse_field(b"1.26  ").unwrap_or(datfile::Field::Blank));
    assert_eq!(datfile::Field::Float(1.),datfile::parse_field(b" 1. ").unwrap_or(datfile::Field::Blank));
    assert_eq!(datfile::Field::Float(2.),datfile::parse_field(b" 2.").unwrap_or(datfile::Field::Blank));
    assert_eq!(datfile::Field::Float(3.),datfile::parse_field(b"3.").unwrap_or(datfile::Field::Blank));
    assert_eq!(datfile::Field::Float(4.),datfile::parse_field(b"4. ").unwrap_or(datfile::Field::Blank));
    assert_eq!(datfile::Field::Float(1.23e7),datfile::parse_field(b"1.23e+7").unwrap_or(datfile::Field::Blank));
    assert_eq!(datfile::Field::Float(1.24e7),datfile::parse_field(b"1.24e+7 ").unwrap_or(datfile::Field::Blank));
    assert_eq!(datfile::Field::Float(2.0e7),datfile::parse_field(b"2e+7 ").unwrap_or(datfile::Field::Blank));
    assert_eq!(datfile::Field::Float(1.25e7),datfile::parse_field(b"1.25+7").unwrap_or(datfile::Field::Blank));
    assert_eq!(datfile::Field::Float(1.26e7),datfile::parse_field(b"1.26+7 ").unwrap_or(datfile::Field::Blank));
    assert_eq!(datfile::Field::Float(1.0e7),datfile::parse_field(b"1.+7 ").unwrap_or(datfile::Field::Blank));
    assert_eq!(datfile::Field::Int(123456),datfile::parse_field(b"123456").unwrap_or(datfile::Field::Blank));
    assert_eq!(datfile::Field::Continuation("A B".to_owned()),datfile::parse_field(b"+A B").unwrap_or(datfile::Field::Blank));
    assert_eq!(datfile::Field::String("HI1".to_owned()),datfile::parse_field(b"HI1").unwrap_or(datfile::Field::Blank));
}