extern crate nastran;
use nastran::datfile;

const DATFILE: &'static [u8] = b"\
PARAM,POST,1$ABC
PARAM,WTMASS,0.00259


ABCDEF,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456
";

#[test]
fn comma_separated() {
    assert_eq!(Some(datfile::Deck {
                        cards: vec![datfile::Card {
                                        fields: vec![datfile::Field::String(b"PARAM,POST,1".to_vec())],
                                        comment: Some(b"$ABC".to_vec()),
                                    },
                                    datfile::Card {
                                        fields: vec![datfile::Field::String(b"PARAM,WTMASS,0.00259".to_vec())],
                                        comment: Some(b"".to_vec()),
                                    },
                                    datfile::Card {
                                        fields: vec![datfile::Field::String(b"ABCDEF,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456,123".to_vec())],
                                        comment: Some(b"456,123456".to_vec()),
                                    }],
                    }),
               datfile::parse_buffer(DATFILE))
}

// #[test]
// fn comment() {
//     assert_eq!(b"$ABC",datfile::read_comment(b"$ABC").unwrap().1);
// }

