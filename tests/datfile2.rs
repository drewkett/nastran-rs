extern crate nastran;
use nastran::datfile2;

const DATFILE: &'static [u8] = b"\
PARAM,POST , 1 $ABC
PARAM, WTMASS,0.00259
+,1,2
+a,1,2


ABCDEF,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456
BLAH    123      1.+5   1e2     ABC
GRID*            1100001               0    3.732130e+02    3.329000e+00 ED00013
*ED00013    7.408100e+01               0
                                          0.      0.059  0.      0.059 		1
";

#[test]
fn comma_separated() {
    let res = match datfile2::parse_buffer(DATFILE) {
        Ok(d) => d,
        Err(e) => {
            println!("{:?}", e);
            assert!(false);
            return;
        }
    };
    let mut it = res.cards.into_iter();
    assert_eq!(it.next(),
               Some(datfile2::Card {
                        first: datfile2::Field::String(b"PARAM"),
                        fields: vec![datfile2::Field::String(b"POST"), datfile2::Field::Int(1)],
                        continuation: None,
                        comment: Some(b"$ABC"),
                        is_comma: true,
                        is_double: false,
                        unparsed: None,
                    }));
    assert_eq!(it.next(),
               Some(datfile2::Card {
                        first: datfile2::Field::String(b"PARAM"),
                        fields: vec![datfile2::Field::String(b"WTMASS"),
                                     datfile2::Field::Float(0.00259)],
                        continuation: None,
                        comment: None,
                        is_comma: true,
                        is_double: false,
                        unparsed: None,
                    }));
    assert_eq!(it.next(),
               Some(datfile2::Card {
                        first: datfile2::Field::Continuation(b""),
                        fields: vec![datfile2::Field::Int(1), datfile2::Field::Int(2)],
                        continuation: None,
                        comment: None,
                        is_comma: true,
                        is_double: false,
                        unparsed: None,
                    }));
    assert_eq!(it.next(),
               Some(datfile2::Card {
                        first: datfile2::Field::Continuation(b"a"),
                        fields: vec![datfile2::Field::Int(1), datfile2::Field::Int(2)],
                        continuation: None,
                        comment: None,
                        is_comma: true,
                        is_double: false,
                        unparsed: None,
                    }));
    assert_eq!(it.next(),
               Some(datfile2::Card {
                        first: datfile2::Field::Blank, // Not sure about this
                        fields: vec![],
                        continuation: None,
                        comment: None,
                        is_comma: false,
                        is_double: false,
                        unparsed: None,
                    }));
    assert_eq!(it.next(),
               Some(datfile2::Card {
                        first: datfile2::Field::Blank, // Not sure about this
                        fields: vec![],
                        continuation: None,
                        comment: None,
                        is_comma: false,
                        is_double: false,
                        unparsed: None,
                    }));
    assert_eq!(it.next(),
               Some(datfile2::Card {
                        first: datfile2::Field::String(b"ABCDEF"),
                        fields: vec![datfile2::Field::Int(123456),
                                     datfile2::Field::Int(123456),
                                     datfile2::Field::Int(123456),
                                     datfile2::Field::Int(123456),
                                     datfile2::Field::Int(123456),
                                     datfile2::Field::Int(123456),
                                     datfile2::Field::Int(123456),
                                     datfile2::Field::Int(123456),
                                     datfile2::Field::Int(123456),
                                     datfile2::Field::Int(123456),
                                     datfile2::Field::Int(123)],
                        continuation: None,
                        comment: Some(b"456,123456"),
                        is_comma: true,
                        is_double: false,
                        unparsed: None,
                    }));
    assert_eq!(it.next(),
               Some(datfile2::Card {
                        first: datfile2::Field::String(b"BLAH"),
                        fields: vec![datfile2::Field::Int(123),
                                     datfile2::Field::Float(1e5),
                                     datfile2::Field::Float(1e2),
                                     datfile2::Field::String(b"ABC")],
                        continuation: None,
                        comment: None,
                        is_comma: false,
                        is_double: false,
                        unparsed: None,
                    }));
    assert_eq!(it.next(),
               Some(datfile2::Card {
                        first: datfile2::Field::DoubleString(b"GRID"),
                        fields: vec![datfile2::Field::Int(1100001),
                                     datfile2::Field::Int(0),
                                     datfile2::Field::Float(373.213),
                                     datfile2::Field::Float(3.329)],
                        continuation: Some(datfile2::Field::Continuation(b"ED00013")),
                        comment: None,
                        is_comma: false,
                        is_double: true,
                        unparsed: None,
                    }));
    assert_eq!(it.next(),
               Some(datfile2::Card {
                        first: datfile2::Field::DoubleContinuation(b"ED00013"),
                        fields: vec![datfile2::Field::Float(74.081), datfile2::Field::Int(0)],
                        continuation: None,
                        comment: None,
                        is_comma: false,
                        is_double: true,
                        unparsed: None,
                    }));
    assert_eq!(it.next(),
               Some(datfile2::Card {
                        first: datfile2::Field::Blank, // Should be continuation first?
                        fields: vec![datfile2::Field::Blank,
                                     datfile2::Field::Blank,
                                     datfile2::Field::Blank,
                                     datfile2::Field::Blank,
                                     datfile2::Field::Float(0.0),
                                     datfile2::Field::Float(0.059),
                                     datfile2::Field::Float(0.0),
                                     datfile2::Field::Float(0.059)],
                        continuation: Some(datfile2::Field::Blank),
                        comment: None,
                        is_comma: false,
                        is_double: false,
                        unparsed: Some(b"1"),
                    }));
    assert_eq!(it.next(), None);
}
