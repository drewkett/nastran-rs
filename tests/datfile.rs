extern crate nastran;
use nastran::datfile;

const DATFILE: &'static [u8] = b"\
PARAM,POST , 1 $ABC
PARAM, WTMASS,0.00259
+,1,2
+a,1,2


ABCDEF,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456
BLAH    123      1.+5   1e2     ABC
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
                    }));
    assert_eq!(it.next(),
               Some(datfile::Card {
                        fields: vec![datfile::Field::String(b"PARAM"),
                                     datfile::Field::String(b"WTMASS"),
                                     datfile::Field::Float(0.00259)],
                        comment: None,
                    }));
    assert_eq!(it.next(),
               Some(datfile::Card {
                        fields: vec![datfile::Field::Continuation(b""),
                                     datfile::Field::Int(1),
                                     datfile::Field::Int(2)],
                        comment: None,
                    }));
    assert_eq!(it.next(),
               Some(datfile::Card {
                        fields: vec![datfile::Field::Continuation(b"a"),
                                     datfile::Field::Int(1),
                                     datfile::Field::Int(2)],
                        comment: None,
                    }));
    assert_eq!(it.next(),
               Some(datfile::Card {
                        fields: vec![],
                        comment: None,
                    }));
    assert_eq!(it.next(),
               Some(datfile::Card {
                        fields: vec![],
                        comment: None,
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
                    }));
    assert_eq!(it.next(),
               Some(datfile::Card {
                        fields: vec![datfile::Field::String(b"BLAH"),
                                     datfile::Field::Int(123),
                                     datfile::Field::Float(1e5),
                                     datfile::Field::Float(1e2),
                                     datfile::Field::String(b"ABC")],
                        comment: None,
                    }));
}

