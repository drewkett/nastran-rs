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
                    datfile::Field::String("PARAM".to_owned())
                ],
                comment: Some("POST".to_owned())
            }
        ]
    }), datfile::parse_buffer(b"PARAM,POST"))
}
