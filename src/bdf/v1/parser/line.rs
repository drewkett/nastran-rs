#[derive(Debug, PartialEq)]
struct LineParser<'a> {
    buffer: &'a [u8],
    trailing: Option<&'a [u8]>,
    chars_seen: usize,
    fields_seen: usize,
}

impl<'a> LineParser<'a> {
    #[allow(dead_code)]
    fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            trailing: None,
            chars_seen: 0,
            fields_seen: 0,
        }
    }

    #[allow(dead_code)]
    fn next_field(&mut self) -> &'a [u8] {
        let n = self.buffer.len();
        for i in 0..n {
            match self.buffer[i] {
                b'\r' | b'\n' => {
                    let (_field, trailing) = self.buffer.split_at(i);
                    self.trailing = Some(trailing);
                    unimplemented!();
                }
                b'\t' | b',' => {
                    let (_field, trailing) = self.buffer.split_at(i);
                    self.trailing = Some(trailing);
                    unimplemented!();
                }
                _ => unimplemented!(),
            }
        }
        unimplemented!();
    }
}

pub fn parse_line() {}
