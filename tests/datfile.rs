extern crate nastran;
use nastran::datfile;

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
    let res = match datfile::parse_buffer(DATFILE) {
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
        Some(datfile::Card {
            first: Some(datfile::Field::String("PARAM")),
            fields: vec![
                datfile::Field::String("POST"),
                datfile::Field::Int(1),
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Blank,
            ],
            continuation: "",
            comment: Some(b"$ABC"),
            is_comma: true,
            is_double: false,
            unparsed: None,
        })
    );
    assert_eq!(
        it.next(),
        Some(datfile::Card {
            first: Some(datfile::Field::String("PARAM")),
            fields: vec![
                datfile::Field::String("WTMASS"),
                datfile::Field::Float(0.00259),
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Int(1),
                datfile::Field::Int(2),
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Blank,
            ],
            continuation: "",
            comment: None,
            is_comma: true,
            is_double: false,
            unparsed: None,
        })
    );
    assert_eq!(
        it.next(),
        Some(datfile::Card {
            first: Some(datfile::Field::String("ABCDEF")),
            fields: vec![
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
                datfile::Field::Int(123),
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Blank,
            ],
            continuation: "",
            comment: Some(b"456,123456"),
            is_comma: true,
            is_double: false,
            unparsed: None,
        })
    );
    assert_eq!(
        it.next(),
        Some(datfile::Card {
            first: Some(datfile::Field::String("BLAH")),
            fields: vec![
                datfile::Field::Int(123),
                datfile::Field::Float(1e5),
                datfile::Field::Float(1e2),
                datfile::Field::String("ABC"),
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Blank,
            ],
            continuation: "",
            comment: None,
            is_comma: false,
            is_double: false,
            unparsed: None,
        })
    );
    assert_eq!(
        it.next(),
        Some(datfile::Card {
            first: Some(datfile::Field::DoubleString("GRID")),
            fields: vec![
                datfile::Field::Int(1100001),
                datfile::Field::Int(0),
                datfile::Field::Float(373.213),
                datfile::Field::Float(3.329),
                datfile::Field::Float(74.081),
                datfile::Field::Int(0),
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Blank,
                datfile::Field::Float(0.0),
                datfile::Field::Float(0.059),
                datfile::Field::Float(0.0),
                datfile::Field::Float(0.059),
            ],
            continuation: "",
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
    let res = match datfile::parse_buffer(HEADER) {
        Ok(d) => d,
        Err(e) => {
            println!("{:?}", e);
            assert!(false);
            return;
        }
    };
    let mut deck = datfile::Deck::new();
    deck.set_header(b"$ABC\n\nNASTRAN\n");
    deck.set_unparsed(b"BCD,2\n");
    deck.add_card(datfile::Card {
        first: Some(datfile::Field::String("ABC")),
        fields: vec![
            datfile::Field::Int(1),
            datfile::Field::Blank,
            datfile::Field::Blank,
            datfile::Field::Blank,
            datfile::Field::Blank,
            datfile::Field::Blank,
            datfile::Field::Blank,
            datfile::Field::Blank,
        ],
        continuation: "",
        comment: None,
        is_comma: true,
        is_double: false,
        unparsed: None,
    });
    assert_eq!(res, deck);
}
