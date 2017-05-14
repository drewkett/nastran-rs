extern crate nastran;
use nastran::datfile;

const DATFILE: &'static [u8] = b"\
PARAM,POST,1$ABC
PARAM,WTMASS,0.00259


ABCDEF,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456
";

#[test]
fn comma_separated() {
    assert_eq!(datfile::Deck {
                   cards: vec![datfile::Card {
                                   fields: vec![datfile::Field::String(b"PARAM".to_vec()),
                                                datfile::Field::String(b"POST".to_vec()),
                                                datfile::Field::Int(1)],
                                   comment: Some(b"$ABC".to_vec()),
                               },
                               datfile::Card {
                                   fields: vec![datfile::Field::String(b"PARAM".to_vec()),
                                                datfile::Field::String(b"WTMASS".to_vec()),
                                                datfile::Field::Float(0.00259)],
                                   comment: Some(b"".to_vec()),
                               },
                               datfile::Card {
                                   fields: vec![datfile::Field::String(b"ABCDEF".to_vec()),
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
               datfile::parse_buffer(DATFILE).unwrap_or(datfile::Deck { cards: vec![] }))
}

// #[test]
// fn comment() {
//     assert_eq!(b"$ABC",datfile::read_comment(b"$ABC").unwrap().1);
// }

