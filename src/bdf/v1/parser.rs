use bstr::ByteSlice;
use std::fmt;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Embedded Space in field")]
    EmbeddedSpace,
    #[error("Unexpected character {}",(&[*.0][..]).as_bstr())]
    UnexpectedChar(u8),
    #[error("Text field greater than 8 chars")]
    TextTooLong,
    #[error("Field is not valid")]
    InvalidField,
    #[error("Error reading datfile : {0}")]
    IO(#[from] io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

struct SplitLines<I> {
    iter: I,
}

impl<I> Iterator for SplitLines<I>
where
    I: Iterator<Item = io::Result<u8>> + Sized,
{
    type Item = Result<Vec<u8>>;

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
                Some(Err(e)) => return Some(Err(e.into())),
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
            Some((_, c @ b'a'..=b'z')) => Some(c - 32),
            Some((_, c)) => Some(c),
            None => None,
        }
    }
}

#[derive(Debug)]
pub struct UnparsedFirstField([u8; 8]);
#[derive(Debug)]
pub struct UnparsedSingleField([u8; 8]);
#[derive(Debug)]
pub struct UnparsedDoubleField([u8; 16]);
#[derive(Debug)]
pub struct UnparsedTrailingField([u8; 8]);

pub enum UnparsedFieldData {
    Blank,
    Single(
        UnparsedFirstField,
        [UnparsedSingleField; 8],
        UnparsedTrailingField,
    ),
    Double(
        UnparsedFirstField,
        [UnparsedDoubleField; 4],
        UnparsedTrailingField,
    ),
}

pub struct UnparsedBulkCard {
    pub original: Vec<u8>,
    data: UnparsedFieldData,
}

impl fmt::Display for UnparsedBulkCard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.data {
            UnparsedFieldData::Blank => write!(f, "\n"),
            UnparsedFieldData::Single(first, fields, trailing) => {
                write!(f, "{}", first.0.as_bstr())?;
                for field in fields.iter() {
                    write!(f, "{}", field.0.as_bstr())?;
                }
                write!(f, "{}", trailing.0.as_bstr())
            }
            UnparsedFieldData::Double(first, fields, trailing) => {
                write!(f, "{}", first.0.as_bstr())?;
                for field in fields.iter() {
                    write!(f, "{}", field.0.as_bstr())?;
                }
                write!(f, "{}", trailing.0.as_bstr())
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
    type Item = Result<UnparsedBulkCard>;

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
                Some(Ok(UnparsedBulkCard {
                    original,
                    data: UnparsedFieldData::Double(
                        UnparsedFirstField(first),
                        [
                            UnparsedDoubleField(field1),
                            UnparsedDoubleField(field2),
                            UnparsedDoubleField(field3),
                            UnparsedDoubleField(field4),
                        ],
                        UnparsedTrailingField(trailing),
                    ),
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
                Some(Ok(UnparsedBulkCard {
                    original,
                    data: UnparsedFieldData::Single(
                        UnparsedFirstField(first),
                        [
                            UnparsedSingleField(field1),
                            UnparsedSingleField(field2),
                            UnparsedSingleField(field3),
                            UnparsedSingleField(field4),
                            UnparsedSingleField(field5),
                            UnparsedSingleField(field6),
                            UnparsedSingleField(field7),
                            UnparsedSingleField(field8),
                        ],
                        UnparsedTrailingField(trailing),
                    ),
                }))
            }
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum FirstFieldKind {
    Blank,
    Text([u8; 8]),
    Continuation([u8; 8]),
}

#[derive(Debug)]
pub struct FirstField {
    kind: FirstFieldKind,
    double: bool,
}

#[derive(Debug)]
pub struct TrailingField([u8; 8]);

#[derive(Debug)]
pub enum Field {
    Blank,
    Int(i32),
    Float(f32),
    Double(f64),
    Text([u8; 8]),
}

#[derive(Debug)]
pub enum FieldData {
    Blank,
    Single(FirstField, [Field; 8], UnparsedTrailingField),
    Double(FirstField, [Field; 4], UnparsedTrailingField),
}

pub struct BulkCard {
    pub original: Vec<u8>,
    pub data: FieldData,
}

fn parse_first_field<I>(field: &mut I) -> Result<FirstField>
where
    I: Iterator<Item = u8>,
{
    enum State {
        Start,
        Blank,
        Alpha,
        Continuation,
        EndAlpha,
        EndContinuation,
    }
    enum ZeroOneTwo {
        Zero,
        One(u8),
        Two(u8, u8),
    }
    use State::*;
    use ZeroOneTwo::*;
    let mut state = State::Start;
    let mut contents = [b' '; 16];
    let mut i = 0;
    let mut double = false;
    while let Some(c) = field.next() {
        let (s, c) = match (state, c, i) {
            (Start, b' ', _) => (Blank, Zero),
            (Start, c @ b'A'..=b'Z', _) => (Alpha, One(c)),
            (Start, b'+', _) => (Continuation, One(b'+')),
            (Start, b'*', _) => {
                double = true;
                (Continuation, One(b'+'))
            }
            (Blank, b' ', _) => (Blank, Zero),
            (Alpha, c @ b'A'..=b'Z', _) => (Alpha, One(c)),
            (Alpha, c @ b'0'..=b'9', _) => (Alpha, One(c)),
            (Alpha, c @ b' ', _) => (EndAlpha, One(c)),
            (Continuation, c @ b'A'..=b'Z', _) => (Continuation, One(c)),
            (Continuation, c @ b'0'..=b'9', _) => (Continuation, One(c)),
            (Continuation, c @ b' ', 0..=7) => (Continuation, One(c)),
            (Continuation, b' ', 8..=usize::MAX) => (Continuation, Zero),
            (EndAlpha, b' ', _) => (EndAlpha, Zero),
            (EndAlpha, b'*', _) => {
                double = true;
                (EndAlpha, Zero)
            }
            (_, c, _) => return Err(Error::UnexpectedChar(c)),
        };
        state = s;
        match c {
            Zero => {}
            One(c1) => {
                contents[i] = c1;
                i += 1;
            }
            Two(c1, c2) => {
                contents[i] = c1;
                i += 1;
                contents[i] = c2;
                i += 1;
            }
        }
    }
    if i > 8 {
        return Err(Error::TextTooLong);
    }
    let mut result = [b' '; 8];
    result[..i].copy_from_slice(&contents[..i]);
    let kind = match state {
        Start | Blank => FirstFieldKind::Blank,
        Alpha | EndAlpha => FirstFieldKind::Text(result),
        Continuation | EndContinuation => FirstFieldKind::Continuation(result),
    };
    Ok(FirstField { kind, double })
}

fn parse_inner_field<I>(field: &mut I) -> Result<Field>
where
    I: Iterator<Item = u8>,
{
    use std::convert::TryInto;
    enum State {
        Start,
        PlusMinus,   // Seen starting '+' or '-'
        Period,      // Seen starting numbers
        FloatPeriod, // Seen starting numbers
        Digits,      // Seen starting '.'
        Alpha,       // Seen starting 'A' to 'Z'
        FloatExponent,
        DoubleExponent,
        FloatSignedExponent,
        DoubleSignedExponent,
        FloatSignedExponentValue,
        DoubleSignedExponentValue,
        EndText,
        EndInt,
        EndFloat,
        EndDouble,
    }
    enum ZeroOneTwo {
        Zero,
        One(u8),
        Two(u8, u8),
    }
    use State::*;
    use ZeroOneTwo::*;
    let mut state = State::Start;
    let mut contents = [b' '; 16];
    let mut i = 0;
    while let Some(c) = field.next() {
        let (s, c) = match (state, c, i) {
            (Start, b' ', _) => (Start, Zero),
            (Start, c @ b'A'..=b'Z', _) => (Alpha, One(c)),
            (Start, b'+', _) => (PlusMinus, Zero),
            (Start, c @ b'-', _) => (PlusMinus, One(c)),
            (Start, c @ b'.', _) => (Period, One(c)),
            (Start, c @ b'0'..=b'9', _) => (Digits, One(c)),
            (Digits, c @ b'0'..=b'9', _) => (Digits, One(c)),
            (Digits, c @ b'.', _) => (FloatPeriod, One(c)),
            (Digits, b' ', _) => (EndInt, Zero),
            // Can't remember if these are valid
            // (Digits, c @ b'E', _) => (FloatPeriod, [*c].iter()),
            // (Digits, c @ b'+', _) => (FloatPeriod, [*c].iter()),
            // (Digits, c @ b'-', _) => (FloatPeriod, [*c].iter()),
            (PlusMinus, c @ b'0'..=b'9', _) => (Digits, One(c)),
            // Can't remember if this is valid
            // (PlusMinus, c @ b'.', _) => (FloatPeriod, [*c].iter()),
            (Period, c @ b'0'..=b'9', _) => (FloatPeriod, One(c)),
            (FloatPeriod, c @ b'0'..=b'9', _) => (FloatPeriod, One(c)),
            (FloatPeriod, b'D', _) => (DoubleExponent, One(b'E')),
            (FloatPeriod, c @ b'E', _) => (FloatExponent, One(c)),
            (FloatPeriod, c @ b'+', _) => (FloatSignedExponent, Two(b'E', c)),
            (FloatPeriod, c @ b'-', _) => (FloatSignedExponent, Two(b'E', c)),
            (FloatPeriod, b' ', _) => (EndFloat, Zero),
            (FloatSignedExponent, c @ b'0'..=b'9', _) => (FloatSignedExponentValue, One(c)),
            (DoubleSignedExponent, c @ b'0'..=b'9', _) => (DoubleSignedExponentValue, One(c)),
            (FloatSignedExponentValue, c @ b'0'..=b'9', _) => (FloatSignedExponentValue, One(c)),
            (DoubleSignedExponentValue, c @ b'0'..=b'9', _) => (DoubleSignedExponentValue, One(c)),
            (FloatSignedExponentValue, b' ', _) => (EndFloat, Zero),
            (DoubleSignedExponentValue, b' ', _) => (EndDouble, Zero),
            (Alpha, c @ b'A'..=b'Z', 0..=7) => (Alpha, One(c)),
            (Alpha, c @ b'0'..=b'9', 0..=7) => (Alpha, One(c)),
            (Alpha, b' ', _) => (EndText, Zero),
            (Alpha, _, 8..=usize::MAX) => return Err(Error::TextTooLong),
            (EndInt, b' ', _) => (EndInt, Zero),
            (EndFloat, b' ', _) => (EndFloat, Zero),
            (EndDouble, b' ', _) => (EndDouble, Zero),
            (EndText, b' ', _) => (EndText, Zero),
            (EndInt, _, _) => return Err(Error::EmbeddedSpace),
            (EndFloat, _, _) => return Err(Error::EmbeddedSpace),
            (EndDouble, _, _) => return Err(Error::EmbeddedSpace),
            (EndText, _, _) => return Err(Error::EmbeddedSpace),
            (_, c, _) => return Err(Error::UnexpectedChar(c)),
        };
        state = s;
        match c {
            Zero => {}
            One(c1) => {
                contents[i] = c1;
                i += 1;
            }
            Two(c1, c2) => {
                contents[i] = c1;
                i += 1;
                contents[i] = c2;
                i += 1;
            }
        }
    }
    match state {
        Start => Ok(Field::Blank),
        PlusMinus | Period | FloatExponent | FloatSignedExponent | DoubleExponent
        | DoubleSignedExponent => Err(Error::InvalidField),
        Digits | EndInt => {
            if i <= 8 {
                Ok(Field::Int(
                    unsafe { std::str::from_utf8_unchecked(&contents[..i]) }
                        .parse()
                        .unwrap(),
                ))
            } else {
                Err(Error::InvalidField)
            }
        }
        FloatPeriod | EndFloat | FloatSignedExponentValue => Ok(Field::Float(
            unsafe { std::str::from_utf8_unchecked(&contents[..i]) }
                .parse()
                .unwrap(),
        )),
        EndDouble | DoubleSignedExponentValue => Ok(Field::Double(
            unsafe { std::str::from_utf8_unchecked(&contents[..i]) }
                .parse()
                .unwrap(),
        )),
        Alpha | EndText => Ok(Field::Text(contents[..8].try_into().unwrap())),
    }
}

fn parse_trailing_field<I>(field: &mut I) -> Result<TrailingField>
where
    I: Iterator<Item = u8>,
{
    enum State {
        Start,
        Middle,
        Blank,
    }
    enum ZeroOneTwo {
        Zero,
        One(u8),
        Two(u8, u8),
    }
    use State::*;
    use ZeroOneTwo::*;
    let mut state = State::Start;
    let mut contents = [b' '; 16];
    let mut i = 0;
    while let Some(c) = field.next() {
        let (s, c) = match (state, c, i) {
            (Start, b' ', _) => (Blank, Zero),
            (Start, c @ b'A'..=b'Z', _) => (Middle, Two(b'+', c)),
            (Start, c @ b'0'..=b'9', _) => (Middle, Two(b'+', c)),
            (Middle, c @ b'A'..=b'Z', _) => (Middle, One(c)),
            (Middle, c @ b'0'..=b'9', _) => (Middle, One(c)),
            (Blank, b' ', _) => (Blank, Zero),
            (_, c, _) => return Err(Error::UnexpectedChar(c)),
        };
        state = s;
        match c {
            Zero => {}
            One(c1) => {
                contents[i] = c1;
                i += 1;
            }
            Two(c1, c2) => {
                contents[i] = c1;
                i += 1;
                contents[i] = c2;
                i += 1;
            }
        }
    }
    if i > 8 {
        return Err(Error::TextTooLong);
    }
    let mut result = [b' '; 8];
    result[..i].copy_from_slice(&contents[..i]);
    Ok(TrailingField(result))
}

impl std::convert::TryFrom<&UnparsedFirstField> for FirstField {
    type Error = Error;
    fn try_from(field: &UnparsedFirstField) -> Result<Self> {
        parse_first_field(&mut field.0.iter().cloned())
    }
}

impl std::convert::TryFrom<&UnparsedSingleField> for Field {
    type Error = Error;
    fn try_from(field: &UnparsedSingleField) -> Result<Self> {
        parse_inner_field(&mut field.0.iter().cloned())
    }
}

impl std::convert::TryFrom<&UnparsedDoubleField> for Field {
    type Error = Error;
    fn try_from(field: &UnparsedDoubleField) -> Result<Self> {
        parse_inner_field(&mut field.0.iter().cloned())
    }
}

impl std::convert::TryFrom<&UnparsedTrailingField> for TrailingField {
    type Error = Error;
    fn try_from(field: &UnparsedTrailingField) -> Result<Self> {
        parse_trailing_field(&mut field.0.iter().cloned())
    }
}

impl std::convert::TryFrom<UnparsedBulkCard> for BulkCard {
    type Error = Error;
    fn try_from(unparsed: UnparsedBulkCard) -> Result<Self> {
        use std::convert::TryInto;
        let UnparsedBulkCard { original, data } = unparsed;
        let data = match data {
            UnparsedFieldData::Blank => FieldData::Blank,
            UnparsedFieldData::Single(first, fields, trailing) => FieldData::Single(
                (&first).try_into()?,
                [
                    (&fields[0]).try_into()?,
                    (&fields[1]).try_into()?,
                    (&fields[2]).try_into()?,
                    (&fields[3]).try_into()?,
                    (&fields[4]).try_into()?,
                    (&fields[5]).try_into()?,
                    (&fields[6]).try_into()?,
                    (&fields[7]).try_into()?,
                ],
                trailing,
            ),
            UnparsedFieldData::Double(first, fields, trailing) => FieldData::Double(
                (&first).try_into()?,
                [
                    (&fields[0]).try_into()?,
                    (&fields[1]).try_into()?,
                    (&fields[2]).try_into()?,
                    (&fields[3]).try_into()?,
                ],
                trailing,
            ),
        };
        Ok(BulkCard { original, data })
    }
}

pub fn parse_bytes<I>(iter: I) -> Result<Vec<BulkCard>>
where
    I: Iterator<Item = io::Result<u8>>,
{
    BulkCardIter::new(iter)
        .map(|r| r.and_then(std::convert::TryInto::try_into))
        .collect()
}
