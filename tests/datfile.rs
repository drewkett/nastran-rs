extern crate nastran;
use nastran::datfile;

const DATFILE: &'static [u8] = b"\
PARAM,POST , 1 $ABC
PARAM, WTMASS,0.00259
+,1,2
+a,1,2


ABCDEF,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456
BLAH    123      1.+5   1e2     ABC
GRID*            1100001               0    3.732130e+02    3.329000e+00 ED00013
*ED00013    7.408100e+01               0
                                          0.      0.059  0.      0.059 		
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
    assert_eq!(it.next(),
               Some(datfile::Card {
                        fields: vec![datfile::Field::String(b"PARAM"),
                                     datfile::Field::String(b"POST"),
                                     datfile::Field::Int(1)],
                        comment: Some(b"ABC"),
                        is_comma: true,
                        is_double: false,
                    }));
    assert_eq!(it.next(),
               Some(datfile::Card {
                        fields: vec![datfile::Field::String(b"PARAM"),
                                     datfile::Field::String(b"WTMASS"),
                                     datfile::Field::Float(0.00259)],
                        comment: None,
                        is_comma: true,
                        is_double: false,
                    }));
    assert_eq!(it.next(),
               Some(datfile::Card {
                        fields: vec![datfile::Field::Continuation(b""),
                                     datfile::Field::Int(1),
                                     datfile::Field::Int(2)],
                        comment: None,
                        is_comma: true,
                        is_double: false,
                    }));
    assert_eq!(it.next(),
               Some(datfile::Card {
                        fields: vec![datfile::Field::Continuation(b"a"),
                                     datfile::Field::Int(1),
                                     datfile::Field::Int(2)],
                        comment: None,
                        is_comma: true,
                        is_double: false,
                    }));
    assert_eq!(it.next(),
               Some(datfile::Card {
                        fields: vec![],
                        comment: None,
                        is_comma: false,
                        is_double: false,
                    }));
    assert_eq!(it.next(),
               Some(datfile::Card {
                        fields: vec![],
                        comment: None,
                        is_comma: false,
                        is_double: false,
                    }));
    assert_eq!(it.next(),
               Some(datfile::Card {
                        fields: vec![datfile::Field::String(b"ABCDEF"),
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
                        comment: Some(b"456,123456"),
                        is_comma: true,
                        is_double: false,
                    }));
    assert_eq!(it.next(),
               Some(datfile::Card {
                        fields: vec![datfile::Field::String(b"BLAH"),
                                     datfile::Field::Int(123),
                                     datfile::Field::Float(1e5),
                                     datfile::Field::Float(1e2),
                                     datfile::Field::String(b"ABC")],
                        comment: None,
                        is_comma: false,
                        is_double: false,
                    }));
    assert_eq!(it.next(),
               Some(datfile::Card {
                        fields: vec![datfile::Field::String(b"GRID"),
                                     datfile::Field::Int(1100001),
                                     datfile::Field::Int(0),
                                     datfile::Field::Float(373.213),
                                     datfile::Field::Float(3.329),
                                     datfile::Field::Continuation(b" ED00013")],
                        comment: None,
                        is_comma: false,
                        is_double: true,
                    }));
    assert_eq!(it.next(),
               Some(datfile::Card {
                        fields: vec![datfile::Field::Continuation(b"ED00013"),
                                     datfile::Field::Float(74.081),
                                     datfile::Field::Int(0),
                                     ],
                        comment: None,
                        is_comma: false,
                        is_double: true,
                    }));
    assert_eq!(it.next(),
               Some(datfile::Card {
                        fields: vec![datfile::Field::Blank, // Should be continuation first
                                     datfile::Field::Blank,
                                     datfile::Field::Blank,
                                     datfile::Field::Blank,
                                     datfile::Field::Blank,
                                     datfile::Field::Float(0.0),
                                     datfile::Field::Float(0.059),
                                     datfile::Field::Float(0.0),
                                     datfile::Field::Float(0.059),
                                     datfile::Field::Continuation(b"")
                                     ],
                        comment: None,
                        is_comma: false,
                        is_double: false,
                    }));
    assert_eq!(it.next(),None);
}

