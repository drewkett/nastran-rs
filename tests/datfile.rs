extern crate nastran;

const DATFILE: &'static str = "\
PARAM,POST
";

#[test]
fn comma_separated() {
    assert_eq!(nastran::Deck {
        cards: vec![
            nastran::Card {
                fields: vec![
                    nastran::Field::String("PARAM,POST".to_owned())
                ]
            }
        ]
    }, nastran::parse_buffer(b"PARAM,POST"))
}
