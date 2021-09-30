mod lines;

use bstr::ByteSlice;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt;

use crate::bdf::{Error, Result};

use lines::{NastranLine, NastranLineIter};

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Comment(SmallVec<[u8; 8]>);

impl Comment {
    fn new() -> Self {
        Self(Default::default())
    }

    fn push(&mut self, c: u8) {
        self.0.push(c)
    }
}

impl fmt::Display for Comment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            Ok(())
        } else if self.0[0] == b'$' {
            write!(f, "{}", self.0.as_bstr())
        } else {
            write!(f, "${}", self.0.as_bstr())
        }
    }
}

impl TryFrom<NastranLine> for UnparsedBulkLine {
    type Error = Error;
    fn try_from(mut line: NastranLine) -> Result<UnparsedBulkLine> {
        let first = parse_first_field(line.take8())?;
        let first = match first {
            Some(field) => field,
            None => {
                if line.end_of_data() {
                    let (comment, eol) = line.comment_and_eol()?;
                    return Ok(UnparsedBulkLine {
                        original: line.original(),
                        comment,
                        eol,
                        data: None,
                    });
                } else {
                    FirstField::default()
                }
            }
        };
        if first.double {
            let field1 = line.take16();
            let field2 = line.take16();
            let field3 = line.take16();
            let field4 = line.take16();
            let trailing = parse_trailing_field(line.take8())?;
            let (comment, eol) = line.comment_and_eol()?;
            Ok(UnparsedBulkLine {
                original: line.original(),
                comment,
                eol,
                data: Some(UnparsedFieldData::Double(
                    first,
                    [
                        UnparsedDoubleField(field1),
                        UnparsedDoubleField(field2),
                        UnparsedDoubleField(field3),
                        UnparsedDoubleField(field4),
                    ],
                    trailing,
                )),
            })
        } else {
            let field1 = line.take8();
            let field2 = line.take8();
            let field3 = line.take8();
            let field4 = line.take8();
            let field5 = line.take8();
            let field6 = line.take8();
            let field7 = line.take8();
            let field8 = line.take8();
            let trailing = parse_trailing_field(line.take8())?;
            let (comment, eol) = line.comment_and_eol()?;
            Ok(UnparsedBulkLine {
                original: line.original(),
                comment,
                eol,
                data: Some(UnparsedFieldData::Single(
                    first,
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
                    trailing,
                )),
            })
        }
    }
}

#[derive(Debug)]
struct CommaField(SmallVec<[u8; 16]>);

impl TryFrom<CommaField> for [u8; 8] {
    type Error = Error;
    fn try_from(field: CommaField) -> Result<Self> {
        if field.0.len() > 8 {
            Err(Error::TextTooLong(field.0.into_vec()))
        } else {
            let mut out = [b' '; 8];
            let n = std::cmp::min(field.0.len(), 8);
            out[..n].copy_from_slice(&field.0[..n]);
            Ok(out)
        }
    }
}

impl TryFrom<CommaField> for [u8; 16] {
    type Error = Error;
    fn try_from(field: CommaField) -> Result<Self> {
        if field.0.len() > 16 {
            Err(Error::TextTooLong(field.0.into_vec()))
        } else {
            let mut out = [b' '; 16];
            let n = std::cmp::min(field.0.len(), 16);
            out[..n].copy_from_slice(&field.0[..n]);
            Ok(out)
        }
    }
}

impl TryFrom<CommaField> for Option<FirstField> {
    type Error = Error;
    fn try_from(field: CommaField) -> Result<Self> {
        if field.0.len() > 8 {
            Err(Error::TextTooLong(field.0.into_vec()))
        } else {
            let mut array = [b' '; 8];
            let n = std::cmp::min(field.0.len(), 8);
            array[..n].copy_from_slice(&field.0[..n]);
            parse_first_field(array)
        }
    }
}

impl TryFrom<CommaField> for UnparsedSingleField {
    type Error = Error;
    fn try_from(field: CommaField) -> Result<Self> {
        field.try_into().map(Self)
    }
}

impl TryFrom<CommaField> for UnparsedDoubleField {
    type Error = Error;
    fn try_from(field: CommaField) -> Result<Self> {
        field.try_into().map(Self)
    }
}

impl TryFrom<CommaField> for ContinuationField {
    type Error = Error;
    fn try_from(field: CommaField) -> Result<Self> {
        if field.0.len() > 8 {
            Err(Error::TextTooLong(field.0.into_vec()))
        } else {
            let mut array = [b' '; 8];
            let n = std::cmp::min(field.0.len(), 8);
            array[..n].copy_from_slice(&field.0[..n]);
            parse_trailing_field(array)
        }
    }
}

struct NastranCommaLine {
    original: Vec<u8>,
    iter: NastranLineIter,
    secondline: bool,
}

impl NastranCommaLine {
    fn new(line: Vec<u8>) -> Self {
        // Add comma check here?
        NastranCommaLine {
            original: line.clone(),
            iter: NastranLineIter::new(line.into_iter()),
            secondline: false,
        }
    }

    fn next_field(&mut self) -> Option<CommaField> {
        use std::iter::FromIterator;
        let mut field = SmallVec::from_iter(
            (&mut self.iter)
                .skip_while(|c| *c == b' ')
                .take_while(|c| *c != b','),
        );
        if field.is_empty() {
            if self.iter.peek().is_none() {
                None
            } else {
                Some(CommaField(SmallVec::new()))
            }
        } else {
            let mut j = field.len();
            while j > 0 && field[j - 1] == b' ' {
                j -= 1;
            }
            field.truncate(j);
            Some(CommaField(field))
        }
    }

    fn next_single_field(&mut self) -> Result<UnparsedSingleField> {
        self.next_field()
            .map(TryInto::try_into)
            .unwrap_or(Ok(UnparsedSingleField([b' '; 8])))
    }

    fn next_double_field(&mut self) -> Result<UnparsedDoubleField> {
        self.next_field()
            .map(TryInto::try_into)
            .unwrap_or(Ok(UnparsedDoubleField([b' '; 16])))
    }

    fn next_trailing_field(&mut self) -> Result<ContinuationField> {
        match self.iter.peek() {
            Some(b'+') | Some(b'\r') | Some(b'\n') => self
                .next_field()
                .map(TryInto::try_into)
                .unwrap_or(Ok(ContinuationField([b' '; 7]))),
            _ => Ok(ContinuationField([b' '; 7])),
        }
    }

    fn comment_and_eol(&mut self) -> Option<(Comment, Option<Eol>)> {
        self.iter.comment_and_eol()
    }
}

impl Iterator for NastranCommaLine {
    type Item = Result<UnparsedBulkLine>;

    fn next(&mut self) -> Option<Self::Item> {
        let first = self.next_field();
        if first.is_none() {
            if let Some((comment, eol)) = self.comment_and_eol() {
                let original = std::mem::take(&mut self.original);
                return Some(Ok(UnparsedBulkLine {
                    original,
                    comment,
                    eol,
                    data: None,
                }));
            } else {
                return None;
            }
        }
        let res = move || -> Self::Item {
            if self.secondline {
                let field1 = first.unwrap().try_into()?;
                let first = FirstField {
                    kind: FirstFieldKind::Continuation(Default::default()),
                    double: false,
                };
                let field2 = self.next_single_field()?;
                let field3 = self.next_single_field()?;
                let field4 = self.next_single_field()?;
                let field5 = self.next_single_field()?;
                let field6 = self.next_single_field()?;
                let field7 = self.next_single_field()?;
                let field8 = self.next_single_field()?;
                let trailing = self.next_trailing_field()?;
                let mut original = vec![];
                let (comment, eol) = match self.comment_and_eol() {
                    Some((comment, eol)) => {
                        std::mem::swap(&mut original, &mut self.original);
                        (comment, eol)
                    }
                    None => (Default::default(), None),
                };
                Ok(UnparsedBulkLine {
                    original,
                    comment,
                    eol,
                    data: Some(UnparsedFieldData::Single(
                        first,
                        [
                            field1, field2, field3, field4, field5, field6, field7, field8,
                        ],
                        trailing,
                    )),
                })
            } else {
                self.secondline = true;
                let first: Option<FirstField> = first.unwrap().try_into()?;
                let first = first.unwrap_or_default();
                if first.double {
                    let field1 = self.next_double_field()?;
                    let field2 = self.next_double_field()?;
                    let field3 = self.next_double_field()?;
                    let field4 = self.next_double_field()?;
                    let trailing = self.next_trailing_field()?;
                    let mut original = vec![];
                    let (comment, eol) = match self.comment_and_eol() {
                        Some((comment, eol)) => {
                            std::mem::swap(&mut original, &mut self.original);
                            (comment, eol)
                        }
                        None => (Default::default(), None),
                    };

                    Ok(UnparsedBulkLine {
                        original,
                        comment,
                        eol,
                        data: Some(UnparsedFieldData::Double(
                            first,
                            [field1, field2, field3, field4],
                            trailing,
                        )),
                    })
                } else {
                    let field1 = self.next_single_field()?;
                    let field2 = self.next_single_field()?;
                    let field3 = self.next_single_field()?;
                    let field4 = self.next_single_field()?;
                    let field5 = self.next_single_field()?;
                    let field6 = self.next_single_field()?;
                    let field7 = self.next_single_field()?;
                    let field8 = self.next_single_field()?;
                    let trailing = self.next_trailing_field()?;
                    let mut original = vec![];
                    let (comment, eol) = match self.comment_and_eol() {
                        Some((comment, eol)) => {
                            std::mem::swap(&mut original, &mut self.original);
                            (comment, eol)
                        }
                        None => (Default::default(), None),
                    };
                    Ok(UnparsedBulkLine {
                        original,
                        comment,
                        eol,
                        data: Some(UnparsedFieldData::Single(
                            first,
                            [
                                field1, field2, field3, field4, field5, field6, field7, field8,
                            ],
                            trailing,
                        )),
                    })
                }
            }
        }();
        Some(res)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Eol {
    CrLf,
    Lf,
}

impl Default for Eol {
    fn default() -> Self {
        Eol::CrLf
    }
}

impl fmt::Display for Eol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[allow(clippy::write_with_newline)]
        match self {
            Self::CrLf => write!(f, "\r\n"),
            Self::Lf => writeln!(f),
        }
    }
}

#[derive(Debug)]
pub struct UnparsedSingleField([u8; 8]);
#[derive(Debug)]
pub struct UnparsedDoubleField([u8; 16]);

#[derive(Debug)]
pub enum UnparsedFieldData {
    Single(FirstField, [UnparsedSingleField; 8], ContinuationField),
    Double(FirstField, [UnparsedDoubleField; 4], ContinuationField),
}

impl std::convert::TryFrom<UnparsedFieldData> for (FirstField, Vec<Field>, ContinuationField) {
    type Error = Error;
    fn try_from(field: UnparsedFieldData) -> Result<Self> {
        match field {
            UnparsedFieldData::Single(first, fields, trailing) => Ok((
                first,
                vec![
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
            )),
            UnparsedFieldData::Double(first, fields, trailing) => Ok((
                first,
                vec![
                    (&fields[0]).try_into()?,
                    (&fields[1]).try_into()?,
                    (&fields[2]).try_into()?,
                    (&fields[3]).try_into()?,
                ],
                trailing,
            )),
        }
    }
}

#[derive(Debug)]
pub struct UnparsedBulkLine {
    pub original: Vec<u8>,
    comment: Comment,
    eol: Option<Eol>,
    data: Option<UnparsedFieldData>,
}

impl fmt::Display for UnparsedBulkLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.data {
            Some(UnparsedFieldData::Single(first, fields, trailing)) => {
                write!(f, "{}", first)?;
                for field in fields.iter() {
                    write!(f, "{}", field.0.as_bstr())?;
                }
                write!(f, "{}", trailing)
            }
            Some(UnparsedFieldData::Double(first, fields, trailing)) => {
                write!(f, "{}", first)?;
                for field in fields.iter() {
                    write!(f, "{}", field.0.as_bstr())?;
                }
                write!(f, "{}", trailing)
            }
            None => writeln!(f),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CardType([u8; 7]);

impl Default for CardType {
    fn default() -> Self {
        Self(*b"       ")
    }
}
// TODO this should be an implementation detail and not exposed because its
// a bit weird to use display width like this
impl fmt::Display for CardType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let width = match f.width() {
            Some(8) | None => 8,
            Some(16) => 16,
            Some(_) => return Err(fmt::Error),
        };
        if width == 8 {
            write!(f, "{} ", self.0.as_bstr())
        } else if width == 16 {
            write!(f, "{}*", self.0.as_bstr())
        } else {
            unreachable!()
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FirstFieldKind {
    Text(CardType),
    Continuation(ContinuationField),
}

impl Default for FirstFieldKind {
    fn default() -> Self {
        FirstFieldKind::Continuation(Default::default())
    }
}

// TODO this should be an implementation detail and not exposed because its
// a bit weird to use display width like this
impl fmt::Display for FirstFieldKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let width = match f.width() {
            Some(8) | None => 8,
            Some(16) => 16,
            Some(_) => return Err(fmt::Error),
        };
        if width == 8 {
            match *self {
                FirstFieldKind::Text(text) => write!(f, "{} ", text.0.as_bstr()),
                FirstFieldKind::Continuation(continuation) => {
                    write!(f, "+{}", continuation.0.as_bstr())
                }
            }
        } else if width == 16 {
            match *self {
                FirstFieldKind::Text(text) => write!(f, "{}*", text.0.as_bstr()),
                FirstFieldKind::Continuation(continuation) => {
                    write!(f, "*{}", continuation.0.as_bstr())
                }
            }
        } else {
            unreachable!()
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FirstField {
    kind: FirstFieldKind,
    double: bool,
}

impl fmt::Display for FirstField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use FirstFieldKind::*;
        match &self {
            FirstField {
                kind: Text(CardType(t)),
                double: false,
            } => write!(f, "{} ", t.as_bstr()),
            FirstField {
                kind: Text(CardType(t)),
                double: true,
            } => write!(f, "{} ", t.as_bstr()),
            FirstField {
                kind: Continuation(ContinuationField(t)),
                double: false,
            } => write!(f, "+{}", t.as_bstr()),
            FirstField {
                kind: Continuation(ContinuationField(t)),
                double: true,
            } => write!(f, "*{}", t.as_bstr()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ContinuationField([u8; 7]);

impl Default for ContinuationField {
    fn default() -> Self {
        Self([b' '; 7])
    }
}

// TODO this should be an implementation detail and not exposed because its
// a bit weird to use display width like this
impl fmt::Display for ContinuationField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let width = match f.width() {
            Some(8) | None => 8,
            Some(16) => 16,
            Some(_) => return Err(fmt::Error),
        };
        if width == 8 {
            write!(f, "+{}", self.0.as_bstr())
        } else if width == 16 {
            write!(f, "*{}", self.0.as_bstr())
        } else {
            unreachable!()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Field {
    Blank,
    Int(i32),
    IntOrId(u32),
    Float(f32),
    Double(f64),
    Text([u8; 8]),
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let width = match f.width() {
            Some(8) | None => 8,
            Some(16) => 16,
            Some(_) => return Err(fmt::Error),
        };
        if width == 8 {
            match *self {
                Field::Blank => write!(f, "        "),
                Field::Int(i) => write!(f, "{:<8}", i),
                Field::IntOrId(i) => write!(f, "{:<8}", i),
                Field::Float(d) => write!(f, "{:8}", float_to_8(d)),
                Field::Double(d) => write!(f, "{:8}", float_to_8(d)),
                Field::Text(s) => write!(f, "{:<8}", s.as_bstr()),
            }
        } else if width == 16 {
            match *self {
                Field::Blank => write!(f, "                "),
                Field::Int(i) => write!(f, "{:<16}", i),
                Field::IntOrId(i) => write!(f, "{:<16}", i),
                Field::Float(d) => write!(f, "{:16}", float_to_16(d)),
                Field::Double(d) => write!(f, "{:16}", float_to_16(d)),
                Field::Text(s) => write!(f, "{:<16}", s.as_bstr()),
            }
        } else {
            unreachable!()
        }
    }
}

impl Default for Field {
    fn default() -> Self {
        Field::Blank
    }
}

pub trait FieldConv {
    fn int(&self) -> Result<i32>;
    fn int_or(&self, value: i32) -> Result<i32>;
    fn id(&self) -> Result<u32>;
    fn id_or(&self, value: u32) -> Result<u32>;
    fn maybe_float(&self) -> Result<Option<f64>>;
    fn float(&self) -> Result<f64>;
    fn float_or(&self, value: f64) -> Result<f64>;
    fn dof(&self) -> Result<[bool; 6]>;
}

impl FieldConv for Field {
    fn int(&self) -> Result<i32> {
        match self {
            Field::Int(v) => Ok(*v),
            Field::IntOrId(v) => Ok(*v as i32),
            _ => Err(Error::UnexpectedField("i32", *self)),
        }
    }
    fn int_or(&self, value: i32) -> Result<i32> {
        match self {
            Field::Int(v) => Ok(*v),
            Field::IntOrId(v) => Ok(*v as i32),
            Field::Blank => Ok(value),
            _ => Err(Error::UnexpectedField("i32", *self)),
        }
    }
    fn id(&self) -> Result<u32> {
        match self {
            Field::IntOrId(v) => Ok(*v as u32),
            _ => Err(Error::UnexpectedField("id", *self)),
        }
    }
    fn id_or(&self, value: u32) -> Result<u32> {
        match self {
            Field::IntOrId(v) => Ok(*v),
            Field::Blank => Ok(value),
            _ => Err(Error::UnexpectedField("id", *self)),
        }
    }
    fn maybe_float(&self) -> Result<Option<f64>> {
        match self {
            Field::Blank => Ok(None),
            Field::Float(f) => Ok(Some(*f as f64)),
            Field::Double(d) => Ok(Some(*d)),
            _ => Err(Error::UnexpectedField("maybe_f64", *self)),
        }
    }
    fn float(&self) -> Result<f64> {
        match self {
            Field::Float(f) => Ok(*f as f64),
            Field::Double(d) => Ok(*d),
            _ => Err(Error::UnexpectedField("f64", *self)),
        }
    }
    fn float_or(&self, value: f64) -> Result<f64> {
        match self {
            Field::Float(f) => Ok(*f as f64),
            Field::Double(d) => Ok(*d),
            Field::Blank => Ok(value),
            _ => Err(Error::UnexpectedField("f64", *self)),
        }
    }
    fn dof(&self) -> Result<[bool; 6]> {
        let mut res = [false; 6];
        match self {
            Field::IntOrId(v) => {
                let mut v = *v;
                while v > 0 {
                    let i = (v % 10) as usize;
                    v /= 10;
                    if i == 0 || i > 6 {
                        return Err(Error::UnexpectedDOF(*self));
                    }
                    res[i - 1] = true;
                }
            }
            Field::Blank => {}
            _ => return Err(Error::UnexpectedDOF(*self)),
        }
        Ok(res)
    }
}

impl FieldConv for Option<Field> {
    fn int(&self) -> Result<i32> {
        self.unwrap_or_default().int()
    }
    fn int_or(&self, value: i32) -> Result<i32> {
        self.unwrap_or_default().int_or(value)
    }
    fn id(&self) -> Result<u32> {
        self.unwrap_or_default().id()
    }
    fn id_or(&self, value: u32) -> Result<u32> {
        self.unwrap_or_default().id_or(value)
    }
    fn maybe_float(&self) -> Result<Option<f64>> {
        match self {
            Some(f) => f.maybe_float(),
            None => Ok(None),
        }
    }
    fn float(&self) -> Result<f64> {
        self.unwrap_or_default().float()
    }
    fn float_or(&self, value: f64) -> Result<f64> {
        self.unwrap_or_default().float_or(value)
    }
    fn dof(&self) -> Result<[bool; 6]> {
        match self {
            Some(f) => f.dof(),
            None => Ok(Default::default()),
        }
    }
}

fn float_to_8<T>(f: T) -> String
where
    T: Into<f64> + Copy + fmt::Display + fmt::LowerExp + dtoa::Floating + std::cmp::PartialOrd,
{
    // FIXME: can be improved
    let mut buf = vec![b' '; 8];
    let f = f.into();
    if let Ok(_n) = dtoa::write(&mut buf[..], f) {
        unsafe { String::from_utf8_unchecked(buf) }
    } else {
        let s = if f <= -1e+100 {
            format!("{:<8.1e}", f)
        } else if f < -1e+10 {
            format!("{:<8.2e}", f)
        } else if f < -1e+0 {
            format!("{:<8.3e}", f)
        } else if f < -1e-9 {
            format!("{:<8.2e}", f)
        } else if f < -1e-99 {
            format!("{:<8.1e}", f)
        } else if f < 0.0 {
            format!("{:<8.0e}", f)
        } else if f <= 1e-99 {
            format!("{:<8.1e}", f)
        } else if f <= 1e-9 {
            format!("{:<8.2e}", f)
        } else if f < 1e+0 {
            format!("{:<8.3e}", f)
        } else if f < 1e+10 {
            format!("{:<8.4e}", f)
        } else if f < 1e+100 {
            format!("{:<8.3e}", f)
        } else {
            format!("{:<8.2e}", f)
        };
        if s.len() > 8 {
            panic!("help '{}'", s)
        }
        s
    }
}

fn float_to_16<T>(f: T) -> String
where
    T: Copy + fmt::Display + fmt::LowerExp + dtoa::Floating,
{
    // FIXME: can be improved
    let mut buf = [b' '; 16];
    if let Ok(n) = dtoa::write(&mut buf[..], f) {
        unsafe { String::from_utf8_unchecked(buf[..n].to_vec()) }
    } else {
        let s = format!("{:16.8e}", f);
        if s.len() > 16 {
            panic!("Couldn't write {} in less than 16 chars '{}'", f, s)
        }
        s
    }
}

pub struct BulkLine {
    pub original: Vec<u8>,
    pub comment: Comment,
    pub eol: Option<Eol>,
    pub data: Option<(FirstField, Vec<Field>, ContinuationField)>,
}

enum ZeroOneTwo {
    Zero,
    One(u8),
    Two(u8, u8),
}

fn parse_first_field(field: [u8; 8]) -> Result<Option<FirstField>> {
    enum State {
        Start,
        Blank,
        Alpha,
        Continuation,
        EndAlpha,
    }
    use State::*;
    use ZeroOneTwo::*;
    let mut state = State::Start;
    let mut contents = [b' '; 16];
    let mut i = 0;
    let mut double = false;
    for &c in field.iter() {
        let (s, c) = match (state, c, i) {
            (Start, b' ', _) => (Blank, Zero),
            (Start, c @ b'A'..=b'Z', _) => (Alpha, One(c)),
            (Start, b'+', _) => (Continuation, Zero),
            (Start, b'*', _) => {
                double = true;
                (Continuation, Zero)
            }
            (Blank, b' ', _) => (Blank, Zero),
            (Alpha, c @ b'A'..=b'Z', _) => (Alpha, One(c)),
            (Alpha, c @ b'0'..=b'9', _) => (Alpha, One(c)),
            (Alpha, c @ b' ', _) => (EndAlpha, One(c)),
            (Alpha, b'*', _) => {
                double = true;
                (EndAlpha, Zero)
            }
            (Continuation, c @ b'A'..=b'Z', _) => (Continuation, One(c)),
            (Continuation, c @ b'0'..=b'9', _) => (Continuation, One(c)),
            (Continuation, c @ b' ', 0..=6) => (Continuation, One(c)),
            (Continuation, b' ', 7..=usize::MAX) => (Continuation, Zero),
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
    if i > 7 {
        return Err(Error::TextTooLong(contents[..i].to_vec()));
    }
    let mut result = [b' '; 7];
    result[..i].copy_from_slice(&contents[..i]);
    let kind = match state {
        Start | Blank => return Ok(None),
        Alpha | EndAlpha => FirstFieldKind::Text(CardType(result)),
        Continuation => FirstFieldKind::Continuation(ContinuationField(result)),
    };
    Ok(Some(FirstField { kind, double }))
}

fn parse_inner_field<I>(field: &mut I) -> Result<Field>
where
    I: Iterator<Item = u8>,
{
    #[derive(Debug)]
    enum State {
        Start,
        PlusMinus,
        Period,
        PlusMinusPeriod,
        FloatPeriod,
        IntOrId,
        Int,
        Alpha,
        FloatExponent,
        DoubleExponent,
        FloatSignedExponent,
        DoubleSignedExponent,
        FloatSignedExponentValue,
        DoubleSignedExponentValue,
        EndText,
        EndInt,
        EndIntOrId,
        EndFloat,
        EndDouble,
    }
    use State::*;
    use ZeroOneTwo::*;
    let mut state = State::Start;
    let mut contents = [b' '; 16];
    let mut i = 0;
    for c in field {
        let (s, c) = match (state, c, i) {
            (Start, b' ', _) => (Start, Zero),
            (Start, c @ b'A'..=b'Z', _) => (Alpha, One(c)),
            (Start, b'+', _) => (PlusMinus, Zero),
            (Start, c @ b'-', _) => (PlusMinus, One(c)),
            (Start, c @ b'.', _) => (Period, One(c)),
            (Start, c @ b'0'..=b'9', _) => (IntOrId, One(c)),
            (Int, c @ b'0'..=b'9', _) => (Int, One(c)),
            (Int, c @ b'.', _) => (FloatPeriod, One(c)),
            (Int, b' ', _) => (EndInt, Zero),
            (Int, b'D', _) => (DoubleExponent, One(b'E')),
            (Int, c @ b'E', _) => (FloatExponent, One(c)),
            (Int, c @ b'+', _) => (FloatSignedExponent, Two(b'E',c)),
            (Int, c @ b'-', _) => (FloatSignedExponent, Two(b'E',c)),
            (IntOrId, c @ b'0'..=b'9', _) => (IntOrId, One(c)),
            (IntOrId, c @ b'.', _) => (FloatPeriod, One(c)),
            (IntOrId, b' ', _) => (EndIntOrId, Zero),
            (IntOrId, c @ b'E', _) => (FloatExponent, One(c)),
            (IntOrId, b'D', _) => (DoubleExponent, One(b'E')),
            (IntOrId, c @ b'+', _) => (FloatSignedExponent, Two(b'E',c)),
            (IntOrId, c @ b'-', _) => (FloatSignedExponent, Two(b'E',c)),
            (PlusMinus, c @ b'0'..=b'9', _) => (Int, One(c)),
            (PlusMinus, c @ b'.', _) => (PlusMinusPeriod, One(c)),
            (PlusMinusPeriod, c @ b'0'..=b'9', _) => (FloatPeriod, One(c)),
            (Period, c @ b'0'..=b'9', _) => (FloatPeriod, One(c)),
            (Period, b'D', _) => (DoubleExponent, One(b'E')),
            (Period, c @ b'E', _) => (FloatExponent, One(c)),
            (Period, c @ b'+', _) => (FloatSignedExponent, Two(b'E', c)),
            (Period, c @ b'-', _) => (FloatSignedExponent, Two(b'E', c)),
            (FloatPeriod, c @ b'0'..=b'9', _) => (FloatPeriod, One(c)),
            (FloatPeriod, b'D', _) => (DoubleExponent, One(b'E')),
            (FloatPeriod, c @ b'E', _) => (FloatExponent, One(c)),
            (FloatPeriod, c @ b'+', _) => (FloatSignedExponent, Two(b'E', c)),
            (FloatPeriod, c @ b'-', _) => (FloatSignedExponent, Two(b'E', c)),
            (FloatPeriod, b' ', _) => (EndFloat, Zero),
            (FloatExponent, c @ b'+', _) => (FloatSignedExponent, One(c)),
            (FloatExponent, c @ b'-', _) => (FloatSignedExponent, One(c)),
            (FloatExponent, c @ b'0'..=b'9', _) => (FloatSignedExponentValue, One(c)),
            (DoubleExponent, c @ b'+', _) => (DoubleSignedExponent, One(c)),
            (DoubleExponent, c @ b'-', _) => (DoubleSignedExponent, One(c)),
            (DoubleExponent, c @ b'0'..=b'9', _) => (DoubleSignedExponentValue, One(c)),
            (FloatSignedExponent, c @ b'0'..=b'9', _) => (FloatSignedExponentValue, One(c)),
            (DoubleSignedExponent, c @ b'0'..=b'9', _) => (DoubleSignedExponentValue, One(c)),
            (FloatSignedExponentValue, c @ b'0'..=b'9', _) => (FloatSignedExponentValue, One(c)),
            (DoubleSignedExponentValue, c @ b'0'..=b'9', _) => (DoubleSignedExponentValue, One(c)),
            (FloatSignedExponentValue, b' ', _) => (EndFloat, Zero),
            (DoubleSignedExponentValue, b' ', _) => (EndDouble, Zero),
            (Alpha, c @ b'A'..=b'Z', 0..=7) => (Alpha, One(c)),
            (Alpha, c @ b'0'..=b'9', 0..=7) => (Alpha, One(c)),
            (Alpha, b' ', _) => (EndText, Zero),
            //(Alpha, _, 8..=usize::MAX) => return Err(Error::TextTooLong(),
            (EndInt, b' ', _) => (EndInt, Zero),
            (EndIntOrId, b' ', _) => (EndIntOrId, Zero),
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
        PlusMinus | PlusMinusPeriod | Period | FloatExponent | FloatSignedExponent
        | DoubleExponent | DoubleSignedExponent => Err(Error::InvalidField),
        Int | EndInt => {
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
        IntOrId | EndIntOrId => {
            if i <= 8 {
                Ok(Field::IntOrId(
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
        Alpha | EndText => {
            if i > 8 {
                Err(Error::TextTooLong(contents[..i].to_vec()))
            } else {
                Ok(Field::Text(contents[..8].try_into().unwrap()))
            }
        }
    }
}

fn parse_trailing_field(field: [u8; 8]) -> Result<ContinuationField> {
    enum State {
        Start,
        Middle,
        End,
        Blank,
    }
    use State::*;
    use ZeroOneTwo::*;
    let mut state = State::Start;
    let mut contents = [b' '; 16];
    let mut i = 0;
    for &c in &field {
        let (s, c) = match (state, c, i) {
            // TODO not sure about how to handle this blank
            (Start, b' ', _) => (Blank, Zero),
            (Start, c @ b'A'..=b'Z', _) => (Middle, One(c)),
            (Start, c @ b'0'..=b'9', _) => (Middle, One(c)),
            (Start, b'+', _) => (Middle, Zero),
            (Middle, c @ b'A'..=b'Z', 6) => (End, One(c)),
            (Middle, c @ b'0'..=b'9', 6) => (End, One(c)),
            (Middle, c @ b'A'..=b'Z', _) => (Middle, One(c)),
            (Middle, c @ b'0'..=b'9', _) => (Middle, One(c)),
            (Middle, c @ b' ', _) => (Middle, One(c)),
            (Blank, b' ', _) => (Blank, Zero),
            (End, b' ', _) => (End, Zero),
            (_, c, _) => {
                return Err(Error::UnexpectedChar(c));
            }
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
    if i > 7 {
        return Err(Error::TextTooLong(contents[..i].to_vec()));
    }
    let mut result = [b' '; 7];
    result[..i].copy_from_slice(&contents[..i]);
    Ok(ContinuationField(result))
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

impl std::convert::TryFrom<UnparsedBulkLine> for BulkLine {
    type Error = Error;
    fn try_from(unparsed: UnparsedBulkLine) -> Result<Self> {
        let UnparsedBulkLine {
            original,
            comment,
            eol,
            data,
        } = unparsed;
        let data = match data {
            None => None,
            Some(field) => Some(field.try_into()?),
        };
        Ok(BulkLine {
            original,
            comment,
            eol,
            data,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct BulkCardData {
    first: CardType,
    fields: Vec<Field>,
}

#[derive(Debug, PartialEq)]
pub struct BulkCard {
    data: Option<BulkCardData>,
    comment: Comment,
    eol: Eol,
    original: Vec<u8>,
}

impl BulkCard {
    pub fn original(&self) -> &[u8] {
        &self.original
    }

    pub fn card_type(&self) -> Option<[u8; 7]> {
        self.data.as_ref().map(|d| d.first.0)
    }

    pub fn fields(&self) -> &[Field] {
        match self.data.as_ref() {
            Some(data) => data.fields.as_slice(),
            None => &[],
        }
    }
}

impl fmt::Display for BulkCard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.data {
            Some(BulkCardData { first, fields }) => {
                let mut first = FirstFieldKind::Text(*first);
                let mut fields = &fields[..];
                loop {
                    let n8 = std::cmp::min(8, fields.len());
                    let (next8, fields_) = fields.split_at(n8);
                    fields = fields_;
                    if next8.iter().any(|f| matches!(f, Field::Double(_))) {
                        let n4 = std::cmp::min(4, n8);
                        let (first4, last4) = next8.split_at(n4);
                        write!(f, "{:16}", first)?;
                        for c in first4 {
                            write!(f, "{:16}", c)?;
                        }
                        if n8 > 4 {
                            // Using 8 here makes it output a plus
                            // write!(f, "{:8}{}", ContinuationField::default(), self.eol)?;
                            writeln!(f, "{:8}", ContinuationField::default())?;
                            write!(f, "{:16}", ContinuationField::default())?;
                            for c in last4 {
                                write!(f, "{:16}", c)?;
                            }
                            if fields.is_empty() {
                                // break write!(f, "{:8}{}{}", "", self.comment, self.eol);
                                break writeln!(f, "{:8}{}", "", self.comment);
                            } else {
                                // write!(f, "{:8}{}", ContinuationField::default(), self.eol)?;
                                write!(f, "{:8}", ContinuationField::default())?;
                            }
                        }
                    } else {
                        write!(f, "{:8}", first)?;
                        for c in next8 {
                            write!(f, "{:8}", c)?;
                        }
                        if fields.is_empty() {
                            // break write!(f, "{:8}{}{}", "", self.comment, self.eol);
                            break writeln!(f, "{:8}{}", "", self.comment);
                        } else {
                            // write!(f, "{:8}{}", ContinuationField::default(), self.eol)?;
                            writeln!(f, "{:8}", ContinuationField::default())?;
                        }
                    }
                    first = FirstFieldKind::Continuation(ContinuationField::default());
                }
            }
            // None => write!(f, "{}{}", self.comment, self.eol),
            None => writeln!(f, "{}", self.comment),
        }
    }
}

struct CardState {
    card: BulkCard,
    complete: bool,
}

impl CardState {
    fn complete(&mut self) {
        self.complete = true
    }
}

struct BulkCardIter<I> {
    lines: I,
    counter: usize,
    continuations: HashMap<ContinuationField, usize>,
    deque: std::collections::VecDeque<CardState>,
}

impl<I> BulkCardIter<I> {
    fn new(lines: I) -> Self {
        Self {
            lines,
            continuations: HashMap::new(),
            counter: 0,
            deque: std::collections::VecDeque::new(),
        }
    }

    fn next_counter(&mut self) -> usize {
        let c = self.counter;
        self.counter += 1;
        c
    }

    fn mark_complete(&mut self, i: usize) {
        match self.deque.get_mut(i - (self.counter - self.deque.len())) {
            Some(c) => c.complete = true,
            _ => unreachable!(),
        }
    }

    fn append_partial(&mut self, card: BulkCard) -> usize {
        self.deque.push_back(CardState {
            complete: false,
            card,
        });
        self.next_counter()
    }

    fn append_continuation(
        &mut self,
        continuation: ContinuationField,
        new_fields: &[Field],
        trailing: ContinuationField,
    ) -> Result<()> {
        match self.continuations.remove(&continuation) {
            Some(i) => {
                match self.deque.get_mut(i - (self.counter - self.deque.len())) {
                    Some(CardState {
                        card:
                            BulkCard {
                                data: Some(BulkCardData { fields, .. }),
                                ..
                            },
                        ..
                    }) => fields.extend_from_slice(new_fields),
                    _ => unreachable!(),
                }
                if let Some(i) = self.continuations.insert(trailing, i) {
                    self.mark_complete(i)
                }
                Ok(())
            }
            None => Err(Error::UnmatchedContinuation(continuation.0)),
        }
    }

    fn insert(&mut self, continuation: ContinuationField, partial: BulkCard) {
        let i = self.append_partial(partial);
        if let Some(i) = self.continuations.insert(continuation, i) {
            self.mark_complete(i)
        }
    }

    fn insert_blank(&mut self, original: Vec<u8>, comment: Comment, eol: Option<Eol>) {
        self.deque.push_back(CardState {
            card: BulkCard {
                data: None,
                original,
                comment,
                eol: eol.unwrap_or_default(),
            },
            complete: true,
        });
        self.next_counter();
    }

    fn complete(&mut self) {
        for c in &mut self.deque {
            c.complete();
        }
    }

    fn next_complete(&mut self) -> Option<BulkCard> {
        match self.deque.pop_front() {
            Some(CardState {
                card,
                complete: true,
            }) => Some(card),
            Some(c) => {
                self.deque.push_front(c);
                None
            }
            None => None,
        }
    }
}

impl<I> Iterator for BulkCardIter<I>
where
    I: Iterator<Item = Result<BulkLine>>,
{
    type Item = Result<BulkCard>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.next_complete() {
            return Some(Ok(c));
        }
        while let Some(line) = self.lines.next() {
            let line = match line {
                Ok(line) => line,
                Err(e) => return Some(Err(e)),
            };
            let BulkLine {
                data,
                original,
                comment,
                eol,
            } = line;
            match data {
                Some((first, fields, trailing)) => match first.kind {
                    FirstFieldKind::Text(first) => self.insert(
                        trailing,
                        BulkCard {
                            data: Some(BulkCardData { first, fields }),
                            original,
                            comment,
                            eol: eol.unwrap_or_default(),
                        },
                    ),
                    FirstFieldKind::Continuation(field) => {
                        if let Err(e) = self.append_continuation(field, &fields, trailing) {
                            return Some(Err(e));
                        }
                    }
                },
                None => self.insert_blank(original, comment, eol),
            }
        }
        self.complete();
        self.next_complete().map(Ok)
    }
}

#[cfg(feature = "parallel")]
pub fn parse_file(
    filename: impl AsRef<std::path::Path>,
) -> Result<impl Iterator<Item = Result<BulkCard>>> {
    use rayon::prelude::*;
    let t = std::time::Instant::now();
    let bytes = std::fs::read(filename)?;
    println!("Read file took {} ms", t.elapsed().as_millis());
    let t = std::time::Instant::now();
    let lines = bytes.par_split(|&c| c == b'\n').map(|line| {
        let original = line.to_vec();
        let n = std::cmp::min(original.len(), 10);
        if original[..n].contains(&b',') {
            // FIXME this is currently dropping continuations for commas
            let mut lines = NastranCommaLine::new(line.to_vec())
                .map(|r| r.and_then(TryInto::try_into))
                .collect::<Result<Vec<BulkLine>>>()?;
            Ok(lines.pop().unwrap())
        } else {
            let line: Result<UnparsedBulkLine> = NastranLine::new(line.to_vec()).try_into();
            line.and_then(TryInto::try_into)
        }
    });
    let lines = lines.collect::<Result<Vec<_>>>()?;
    println!("Line parsing took {} ms", t.elapsed().as_millis());
    Ok(BulkCardIter::new(lines.into_iter().map(Ok)))
}

#[cfg(not(feature = "parallel"))]
pub fn parse_file(
    filename: impl AsRef<std::path::Path>,
) -> Result<impl Iterator<Item = Result<BulkCard>>> {
    // FIXME this is awkward. Either the bulk card iter should open the file
    let t = std::time::Instant::now();
    let bytes = std::fs::read(filename)?;
    println!("Read file took {} ms", t.elapsed().as_millis());
    let t = std::time::Instant::now();
    let lines = bytes.split(|&c| c == b'\n').flat_map(|line| {
        let n = std::cmp::min(line.len(), 10);
        if line[..n].contains(&b',') {
            let lines = NastranCommaLine::new(line.to_vec())
                .map(|r| r.and_then(TryInto::try_into))
                .collect::<Result<Vec<BulkLine>>>()?;
            rayon::iter::Either::Left(lines.into_par_iter())
        } else {
            let line: Result<UnparsedBulkLine> = NastranLine::new(line.to_vec()).try_into();
            rayon::iter::Either::Right(rayon::iter::once(line.and_then(TryInto::try_into)))
        }
    });
    println!("Line parsing took {} ms", t.elapsed().as_millis());
    let cards = BulkCardIter::new(lines)
        .collect::<Result<Vec<BulkCard>>>()?
        .into_iter()
        .map(Ok);
    Ok(cards)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_field() {
        assert_eq!(Field::Float(1000.),parse_inner_field(&mut b"1.E3".iter().copied()).unwrap());
        assert_eq!(Field::Float(1000.),parse_inner_field(&mut b"1E3".iter().copied()).unwrap());
        assert_eq!(Field::Float(1000.),parse_inner_field(&mut b"1+3".iter().copied()).unwrap());
        assert_eq!(Field::Double(1000.),parse_inner_field(&mut b"1.D3".iter().copied()).unwrap());
        assert_eq!(Field::Double(1000.),parse_inner_field(&mut b"1D3".iter().copied()).unwrap());
        assert_eq!(Field::Double(1000.),parse_inner_field(&mut b"1D+3".iter().copied()).unwrap());
        //assert_eq!((),parse_inner_field(b"1.D+3"));
        //assert_eq!((),parse_inner_field(b"1.D-3"));
        //assert_eq!((),parse_inner_field(b"1D3"));
        //assert_eq!((),parse_inner_field(b"1D+3"));
        //assert_eq!((),parse_inner_field(b"1D-3"));
    }
}
