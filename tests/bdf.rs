extern crate nastran;
use nastran::bdf::v0::{
    parser::{parse_buffer, Card, Deck},
    Field,
};

const DATFILE: &'static [u8] = b"\
PARAM,POST , 1 $ABC
PARAM, WTMASS,0.00259
+,1,2
ABCDEF,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456
BLAH    123      1.+5   1e2     ABC
GRID*            1100001               0    3.732130e+02    3.329000e+00 ED00013
*ED00013    7.408100e+01               0
                                          0.      0.059  0.      0.059 		1
";

#[test]
fn comma_separated() {
    let res = match parse_buffer(DATFILE) {
        Ok(d) => d,
        Err(e) => {
            println!("{:?}", e);
            assert!(false);
            return;
        }
    };
    let mut it = res.cards.into_iter();
    assert_eq!(
        it.next(),
        Some(Card {
            first: Some(Field::String(*b"PARAM   ")),
            fields: vec![
                Field::String(*b"POST    "),
                Field::Int(1),
                Field::Blank,
                Field::Blank,
                Field::Blank,
                Field::Blank,
                Field::Blank,
                Field::Blank,
            ],
            continuation: *b"        ",
            comment: Some(b"$ABC"),
            is_comma: true,
            is_double: false,
            unparsed: None,
        })
    );
    assert_eq!(
        it.next(),
        Some(Card {
            first: Some(Field::String(*b"PARAM   ")),
            fields: vec![
                Field::String(*b"WTMASS  "),
                Field::Float(0.00259),
                Field::Blank,
                Field::Blank,
                Field::Blank,
                Field::Blank,
                Field::Blank,
                Field::Blank,
                Field::Int(1),
                Field::Int(2),
                Field::Blank,
                Field::Blank,
                Field::Blank,
                Field::Blank,
                Field::Blank,
                Field::Blank,
            ],
            continuation: *b"        ",
            comment: None,
            is_comma: true,
            is_double: false,
            unparsed: None,
        })
    );
    assert_eq!(
        it.next(),
        Some(Card {
            first: Some(Field::String(*b"ABCDEF  ")),
            fields: vec![
                Field::Int(123456),
                Field::Int(123456),
                Field::Int(123456),
                Field::Int(123456),
                Field::Int(123456),
                Field::Int(123456),
                Field::Int(123456),
                Field::Int(123456),
                Field::Int(123456),
                Field::Int(123456),
                Field::Int(123),
                Field::Blank,
                Field::Blank,
                Field::Blank,
                Field::Blank,
                Field::Blank,
            ],
            continuation: *b"        ",
            comment: Some(b"456,123456"),
            is_comma: true,
            is_double: false,
            unparsed: None,
        })
    );
    assert_eq!(
        it.next(),
        Some(Card {
            first: Some(Field::String(*b"BLAH    ")),
            fields: vec![
                Field::Int(123),
                Field::Float(1e5),
                Field::Float(1e2),
                Field::String(*b"ABC     "),
                Field::Blank,
                Field::Blank,
                Field::Blank,
                Field::Blank,
            ],
            continuation: *b"        ",
            comment: None,
            is_comma: false,
            is_double: false,
            unparsed: None,
        })
    );
    assert_eq!(
        it.next(),
        Some(Card {
            first: Some(Field::DoubleString(*b"GRID    ")),
            fields: vec![
                Field::Int(1100001),
                Field::Int(0),
                Field::Float(373.213),
                Field::Float(3.329),
                Field::Float(74.081),
                Field::Int(0),
                Field::Blank,
                Field::Blank,
                Field::Blank,
                Field::Blank,
                Field::Blank,
                Field::Blank,
                Field::Float(0.0),
                Field::Float(0.059),
                Field::Float(0.0),
                Field::Float(0.059),
            ],
            continuation: *b"        ",
            comment: None,
            is_comma: false,
            is_double: true,
            unparsed: None,
        })
    );
    assert_eq!(it.next(), None);
}

const HEADER: &'static [u8] = b"\
$ABC

NASTRAN
BEGIN BULK
ABC,1
ENDDATA
BCD,2
";

#[test]
fn header() {
    let res = match parse_buffer(HEADER) {
        Ok(d) => d,
        Err(e) => {
            println!("{:?}", e);
            assert!(false);
            return;
        }
    };
    let mut deck = Deck::new();
    deck.set_header(b"$ABC\n\nNASTRAN\n");
    deck.set_unparsed(b"BCD,2\n");
    deck.add_card(Card {
        first: Some(Field::String(*b"ABC     ")),
        fields: vec![
            Field::Int(1),
            Field::Blank,
            Field::Blank,
            Field::Blank,
            Field::Blank,
            Field::Blank,
            Field::Blank,
            Field::Blank,
        ],
        continuation: *b"        ",
        comment: None,
        is_comma: true,
        is_double: false,
        unparsed: None,
    });
    assert_eq!(res, deck);
}
