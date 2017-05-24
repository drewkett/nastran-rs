extern crate nastran;
use nastran::datfile;

const DATFILE: &'static [u8] = b"\
PARAM,POST , 1 $ABC
PARAM, WTMASS,0.00259


ABCDEF,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456
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
                               }],
               },
               res)
}

#[test]
fn test_parse() {
    assert_eq!(Field::Float(1.23),parse_field(b" 1.23 ").unwrap_or(Field::Blank));
    assert_eq!(Field::Float(1.),parse_field(b" 1. ").unwrap_or(Field::Blank));
    assert_eq!(Field::Float(1.23e7),parse_field(b"1.23e+7").unwrap_or(Field::Blank));
    assert_eq!(Field::Float(1.25e7),parse_field(b"1.25+7").unwrap_or(Field::Blank));
    assert_eq!(Field::Int(123456),parse_field(b"123456").unwrap_or(Field::Blank));
    assert_eq!(Field::Continuation("A B".to_owned()),parse_field(b"+A B").unwrap_or(Field::Blank));
    assert_eq!(Field::String("HI1".to_owned()),parse_field(b"HI1").unwrap_or(Field::Blank));
}