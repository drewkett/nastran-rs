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

