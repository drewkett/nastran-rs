extern crate nastran;

use nastran::datfile;

const DATFILE: &'static str = "\
PARAM,POST
";

#[test]
fn comma_separated() {
    assert_eq!(datfile::Deck {
        cards: vec![
            datfile::Card {
                fields: vec![
                    datfile::Field::String(b"PARAM".to_vec()),
                    datfile::Field::String(b"POST".to_vec()),
                ],
                comment: None
            }
        ]
    }, datfile::parse_buffer(b"PARAM,POST").unwrap().1)
}
