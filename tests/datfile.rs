extern crate nastran;

use nastran::datfile;

const DATFILE: &'static str = "\
PARAM,POST
";

#[test]
fn comma_separated() {
    assert_eq!(Some(datfile::Deck {
        cards: vec![
            datfile::Card {
                fields: vec![
                    datfile::Field::String(b"PARAM".to_vec())
                ],
                comment: Some(b"POST".to_vec())
            }
        ]
    }), datfile::parse_buffer(b"PARAM,POST"))
}
