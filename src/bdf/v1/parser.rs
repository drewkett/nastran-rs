use bstr::ByteSlice;
use std::fmt;
use std::io;

struct SplitLines<I> {
    iter: I,
}

impl<I> Iterator for SplitLines<I>
where
    I: Iterator<Item = io::Result<u8>> + Sized,
{
    type Item = io::Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result = Vec::with_capacity(80);
        loop {
            match self.iter.next() {
                Some(Ok(c)) => {
                    result.push(c);
                    if c == b'\n' {
                        break;
                    }
                }
                Some(Err(e)) => return Some(Err(e)),
                None => break,
            }
        }
        if result.is_empty() {
            None
        } else {
            Some(Ok(result))
        }
    }
}

trait NastranFileIter: Iterator<Item = io::Result<u8>> + Sized {
    fn split_lines(self) -> SplitLines<Self> {
        SplitLines { iter: self }
    }
}

impl<I> NastranFileIter for I where I: Iterator<Item = io::Result<u8>> + Sized {}

struct ExpandTabs<I> {
    iter: I,
    col: usize,
    seen_tab: bool,
}

impl<I> ExpandTabs<I>
where
    I: Sized,
{
    fn new(iter: I) -> Self {
        ExpandTabs {
            iter,
            col: 0,
            seen_tab: false,
        }
    }
}

impl<I> Iterator for ExpandTabs<I>
where
    I: Iterator<Item = u8>,
{
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.seen_tab && self.col % 8 != 0 {
            self.col += 1;
            return Some(b' ');
        }
        match self.iter.next() {
            Some(b'\t') => {
                self.seen_tab = true;
                self.col += 1;
                return Some(b' ');
            }
            Some(c) => {
                self.col += 1;
                return Some(c);
            }
            None => return None,
        }
    }
}

struct NastranLine {
    original: Vec<u8>,
    iter: NastranLineIter,
}

impl NastranLine {
    fn new(line: Vec<u8>) -> Self {
        // Add comma check here?
        NastranLine {
            original: line.clone(),
            iter: NastranLineIter::new(line.into_iter()),
        }
    }

    fn take8(&mut self) -> [u8; 8] {
        let mut field = [b' '; 8];
        let mut iter = (&mut self.iter)
            .take(8)
            .skip_while(|c| *c == b' ')
            .enumerate();
        while let Some((i, c)) = iter.next() {
            field[i] = c
        }
        field
    }

    fn take16(&mut self) -> [u8; 16] {
        let mut field = [b' '; 16];
        let mut iter = (&mut self.iter)
            .take(16)
            .skip_while(|c| *c == b' ')
            .enumerate();
        while let Some((i, c)) = iter.next() {
            field[i] = c
        }
        field
    }
}

struct NastranLineIter {
    iter: std::iter::Fuse<std::iter::Enumerate<ExpandTabs<std::vec::IntoIter<u8>>>>,
}

impl NastranLineIter {
    fn new(iter: std::vec::IntoIter<u8>) -> Self {
        Self {
            iter: ExpandTabs::new(iter).enumerate().fuse(),
        }
    }
}

impl Iterator for NastranLineIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some((_, b'$')) => None,
            Some((_, b'\n')) => None,
            Some((_, b'\r')) => None,
            Some((80, _)) => None,
            Some((_, c)) => Some(c),
            None => None,
        }
    }
}

pub enum FieldData {
    Blank,
    Single([[u8; 8]; 10]),
    Double([u8; 8], [[u8; 16]; 4], [u8; 8]),
}

pub struct BulkCard {
    pub original: Vec<u8>,
    data: FieldData,
}

impl fmt::Display for BulkCard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.data {
            FieldData::Blank => write!(f, "\n"),
            FieldData::Single(fields) => {
                for field in fields.iter() {
                    write!(f, "{}", field.as_bstr())?;
                }
                Ok(())
            }
            FieldData::Double(first, fields, trailing) => {
                write!(f, "{}", first.as_bstr())?;
                for field in fields.iter() {
                    write!(f, "{}", field.as_bstr())?;
                }
                write!(f, "{}", trailing.as_bstr())
            }
        }
    }
}

struct BulkCardIter<I> {
    iter: SplitLines<I>,
}

impl<I> BulkCardIter<I>
where
    I: Iterator<Item = io::Result<u8>> + Sized,
{
    fn new(iter: I) -> Self {
        Self {
            iter: iter.split_lines(),
        }
    }
}

impl<I> Iterator for BulkCardIter<I>
where
    I: Iterator<Item = io::Result<u8>>,
{
    type Item = io::Result<BulkCard>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(line) = self.iter.next() {
            let line = match line {
                Ok(l) => l,
                Err(e) => return Some(Err(e)),
            };
            let original = line.clone();
            let mut line = NastranLine::new(line);
            let first = line.take8();
            let double = first.contains(&b'*');
            if double {
                let field1 = line.take16();
                let field2 = line.take16();
                let field3 = line.take16();
                let field4 = line.take16();
                let trailing = line.take8();
                Some(Ok(BulkCard {
                    original,
                    data: FieldData::Double(first, [field1, field2, field3, field4], trailing),
                }))
            } else {
                let field1 = line.take8();
                let field2 = line.take8();
                let field3 = line.take8();
                let field4 = line.take8();
                let field5 = line.take8();
                let field6 = line.take8();
                let field7 = line.take8();
                let field8 = line.take8();
                let trailing = line.take8();
                Some(Ok(BulkCard {
                    original,
                    data: FieldData::Single([
                        first, field1, field2, field3, field4, field5, field6, field7, field8,
                        trailing,
                    ]),
                }))
            }
        } else {
            None
        }
    }
}

pub fn parse_bytes<I>(iter: I) -> io::Result<Vec<BulkCard>>
where
    I: Iterator<Item = io::Result<u8>>,
{
    BulkCardIter::new(iter).collect()
}
