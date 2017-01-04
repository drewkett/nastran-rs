
use std::env;
use std::fs::File;
use std::io::Read;

enum Field {
    Blank,
    Int(i32),
    Float(f32),
    Double(f64),
    Continuation(String),
    String(String),
}

struct Card {
    fields: Vec<Field>,
}

struct Deck {
    cards: Vec<Deck>,
}

fn main() {
    let args = env::args().skip(1);
    for arg in args {
        let mut f = File::open(&arg).unwrap();
        let mut buf = vec![];
        let n = f.read_to_end(&mut buf).unwrap();
        println!("{} : {} bytes read", arg, n);
    }
}
