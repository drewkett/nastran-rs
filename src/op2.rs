use std::fmt;
use std::mem;

use bstr::ByteSlice;
use thiserror::Error;

pub trait Word: fmt::Debug + fmt::Display + PartialEq + Copy {}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SingleWord([u8; 4]);

impl fmt::Display for SingleWord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.as_bstr())
    }
}

impl Word for SingleWord {}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DoubleWord([u8; 4], [u8; 4]);

impl fmt::Display for DoubleWord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.as_bstr())
    }
}

impl Word for DoubleWord {}

pub trait Precision: fmt::Debug + Sized + Copy {
    type Int: fmt::Debug + fmt::Display + PartialEq + PartialOrd + Copy + Into<i64> + From<i32>;
    type UInt: fmt::Debug + fmt::Display;
    type Float: fmt::Debug + fmt::Display;
    type Word: Word;

    const WORDSIZE: usize;

    fn zero_int() -> Self::Int;
    fn max_int() -> Self::Int;
    fn max_int_usize() -> usize;
    fn int_from_usize(v: usize) -> Result<Self::Int, ErrorCode<Self>>;
    fn header_code() -> [Self::Word; 7];
    fn i32_from_usize(v: usize) -> Result<i32, ErrorCode<Self>> {
        if v > i32::MAX as usize {
            Err(ErrorCode::ReadTooLarge)
        } else {
            Ok(v as i32)
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SinglePrecision;

impl Precision for SinglePrecision {
    type Int = i32;
    type UInt = u32;
    type Float = f32;
    type Word = SingleWord;

    const WORDSIZE: usize = 4;

    fn zero_int() -> Self::Int {
        0
    }

    fn max_int_usize() -> usize {
        Self::Int::MAX as usize
    }

    fn max_int() -> Self::Int {
        Self::Int::MAX
    }

    fn int_from_usize(v: usize) -> Result<Self::Int, ErrorCode<Self>> {
        if v > Self::max_int_usize() {
            Err(ErrorCode::ReadTooLarge)
        } else {
            Ok(v as Self::Int)
        }
    }
    fn header_code() -> [Self::Word; 7] {
        [
            SingleWord(*b"NAST"),
            SingleWord(*b"RAN "),
            SingleWord(*b"FORT"),
            SingleWord(*b" TAP"),
            SingleWord(*b"E ID"),
            SingleWord(*b" COD"),
            SingleWord(*b"E - "),
        ]
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DoublePrecision;

impl Precision for DoublePrecision {
    type Int = i64;
    type UInt = u64;
    type Float = f64;
    type Word = DoubleWord;

    const WORDSIZE: usize = 8;

    fn zero_int() -> Self::Int {
        0
    }
    fn max_int() -> Self::Int {
        Self::Int::MAX
    }
    fn max_int_usize() -> usize {
        Self::Int::MAX as usize
    }
    fn int_from_usize(v: usize) -> Result<Self::Int, ErrorCode<Self>> {
        if v > Self::max_int_usize() {
            Err(ErrorCode::ReadTooLarge)
        } else {
            Ok(v as Self::Int)
        }
    }
    fn header_code() -> [Self::Word; 7] {
        [
            DoubleWord(*b"NAST", *b"    "),
            DoubleWord(*b"RAN ", *b"    "),
            DoubleWord(*b"FORT", *b"    "),
            DoubleWord(*b" TAP", *b"    "),
            DoubleWord(*b"E ID", *b"    "),
            DoubleWord(*b" COD", *b"    "),
            DoubleWord(*b"E - ", *b"    "),
        ]
    }
}

#[derive(Debug, Error)]
pub enum ErrorCode<P: Precision> {
    #[error("Bytes remaining")]
    BytesRemaining,
    #[error("Unexpected EOF")]
    UnexpectedEOF,
    #[error("Unexpected EOR ({0})")]
    UnexpectedEOR(P::Int),
    #[error("IO Error {0}")]
    IO(#[from] std::io::Error),
    #[error("UnexpectedDataSize: expected={0} found={1}")]
    UnexpectedDataSize(i32, i32),
    #[error("UnexpectedDataLength: expected={0} found={1}")]
    UnexpectedDataLength(usize, usize),
    #[error("UnexpectedValue")]
    UnexpectedValue,
    #[error("Error with negative read : {0}")]
    NegativeRead(i32),
    #[error("Struct too large")]
    StructTooLarge,
    #[error("Read too large")]
    ReadTooLarge,
    #[error("Alignment Error")]
    AlignmentError,
    #[error("Struct not multiple of word size")]
    StructNotWordSizeMultiple,
    #[error("UnknownDataBlockType ({0})")]
    UnknownDataBlockType(i64),
    #[error("ExpectedEOR found {0}")]
    ExpectedEOR(P::Int),
    #[error("Expected EOR found {0:?}")]
    ExpectedEOR2(EncodedSize<P>),
    #[error("ExpectedData")]
    ExpectedData,
    #[error("Integer Conversion")]
    TryFrom(#[from] std::num::TryFromIntError),
}

#[derive(Debug, Error)]
#[error("{0}\nnext bytes:\n{1:?}",code,&remaining[..std::cmp::min(remaining.len(),20)])]
pub struct Error<'a, P: Precision> {
    code: ErrorCode<P>,
    remaining: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date<P: Precision> {
    month: P::Int,
    day: P::Int,
    year: P::Int,
}

#[derive(Debug, PartialEq)]
pub struct FileHeader<P: Precision> {
    date: Date<P>,
    label: [P::Word; 2], // Might want to make this fixed length at some point
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataBlockType {
    Table,
    Matrix,
    StringFactor,
    MatrixFactor,
}

impl DataBlockType {
    fn parse<P: Precision>(v: impl Into<i64>) -> Result<Self, ErrorCode<P>> {
        match v.into() {
            0 => Ok(DataBlockType::Table),
            1 => Ok(DataBlockType::Matrix),
            2 => Ok(DataBlockType::StringFactor),
            3 => Ok(DataBlockType::MatrixFactor),
            n => Err(ErrorCode::UnknownDataBlockType(n)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DataBlock<'a, P: Precision> {
    name: [P::Word; 2],
    trailer: [P::Int; 7],
    record_type: DataBlockType,
    header: &'a [P::Word],
    records: Vec<&'a [u8]>,
}

#[derive(Debug, PartialEq)]
pub struct OP2<'a, P: Precision> {
    header: FileHeader<P>,
    blocks: Vec<DataBlock<'a, P>>,
}

#[derive(Debug)]
pub enum EncodedSize<P: Precision> {
    Data(P::Int),
    Zero,
    EOR(P::Int),
}

pub enum EncodedData<P: Precision, T> {
    Data(T),
    Zero,
    EOR(P::Int),
}

struct OP2Parser<'a, P> {
    buffer: &'a [u8],
    precision: std::marker::PhantomData<P>,
}

impl<'a, P: 'a> OP2Parser<'a, P>
where
    P: Precision,
{
    #[inline]
    fn take(&mut self, n: usize) -> Result<&'a [u8], ErrorCode<P>> {
        if self.buffer.len() < n {
            return Err(ErrorCode::UnexpectedEOF);
        }
        let (ret, buffer) = self.buffer.split_at(n);
        self.buffer = buffer;
        Ok(ret)
    }

    fn read_i32(&mut self) -> Result<i32, ErrorCode<P>> {
        let sl = self.take(4)?;
        let sl = sl.as_ptr() as *const [u8; 4];
        Ok(i32::from_le_bytes(unsafe { *sl }))
    }

    fn read_i32_value(&mut self, expected: i32) -> Result<(), ErrorCode<P>> {
        let found = self.read_i32()?;
        if found != expected {
            return Err(ErrorCode::UnexpectedDataSize(expected, found));
        }
        Ok(())
    }

    fn read_padded<T>(&mut self) -> Result<&'a T, ErrorCode<P>> {
        let expected = mem::size_of::<T>();
        let expected_i = P::i32_from_usize(expected)?;
        self.read_i32_value(expected_i)?;
        let res = self.take(expected)?;
        self.read_i32_value(expected_i)?;
        let res = unsafe { &*(res.as_ptr() as *const T) };
        Ok(res)
    }

    fn read_padded_value<T: PartialEq + fmt::Debug>(
        &mut self,
        expected_value: &T,
    ) -> Result<&'a T, ErrorCode<P>> {
        let value = self.read_padded()?;
        if value != expected_value {
            //eprintln!("{:?} != {:?}", value, expected_value);
            return Err(ErrorCode::UnexpectedValue);
        }
        Ok(value)
    }

    #[allow(dead_code)]
    fn read_padded_byte_slice(&mut self) -> Result<&'a [u8], ErrorCode<P>> {
        let n = self.read_i32()?;
        if n < 1 {
            return Err(ErrorCode::NegativeRead(n));
        }
        let res = self.take(n as usize)?;
        let expected = n;
        let n = self.read_i32()?;
        if n != expected {
            //eprintln!("{:?} != {:?}", n, expected);
            return Err(ErrorCode::UnexpectedValue);
        }
        Ok(res)
    }

    fn read_padded_slice<T>(&mut self) -> Result<&'a [T], ErrorCode<P>> {
        let size = std::mem::size_of::<T>();
        let n = self.read_i32()?;
        if n < 1 {
            return Err(ErrorCode::NegativeRead(n));
        }
        if n as usize % size != 0 {
            return Err(ErrorCode::AlignmentError);
        }
        let n_values = n as usize / size;
        let res = self.take(n as usize)?;
        let expected = n;
        let n = self.read_i32()?;
        if n != expected {
            //eprintln!("{:?} != {:?}", n, expected);
            return Err(ErrorCode::UnexpectedValue);
        }
        let res = unsafe { std::slice::from_raw_parts(res.as_ptr() as *const T, n_values) };
        Ok(res)
    }

    #[allow(dead_code)]
    fn read_encoded_data_byte_slice(&mut self) -> Result<EncodedData<P, &'a [u8]>, ErrorCode<P>> {
        let nwords: P::Int = *self.read_padded()?;
        if nwords < P::zero_int() {
            Ok(EncodedData::EOR(nwords))
        } else if nwords == P::zero_int() {
            Ok(EncodedData::Zero)
        } else {
            let nwords: i64 = nwords.into();
            let nbytes = (nwords as usize) * P::WORDSIZE;
            let ret = self.read_padded_slice()?;
            if ret.len() != nbytes {
                return Err(ErrorCode::UnexpectedDataLength(nbytes, ret.len()));
            }
            Ok(EncodedData::Data(ret))
        }
    }

    fn read_encoded_data_slice<T>(&mut self) -> Result<EncodedData<P, &'a [T]>, ErrorCode<P>> {
        let nwords: P::Int = *self.read_padded()?;
        if nwords < P::zero_int() {
            Ok(EncodedData::EOR(nwords))
        } else if nwords == P::zero_int() {
            Ok(EncodedData::Zero)
        } else {
            let nwords: i64 = nwords.into();
            let nbytes = (nwords as usize) * P::WORDSIZE;
            let size = std::mem::size_of::<T>();
            if nbytes % size != 0 {
                return Err(ErrorCode::AlignmentError);
            }
            let nvalues = nbytes / size;
            let ret = self.read_padded_slice()?;
            if ret.len() != nvalues {
                return Err(ErrorCode::UnexpectedDataLength(nvalues, ret.len()));
            }
            Ok(EncodedData::Data(ret))
        }
    }

    fn read_encoded_data<T>(&mut self) -> Result<EncodedData<P, &'a T>, ErrorCode<P>> {
        let nwords: P::Int = *self.read_padded()?;
        if nwords < P::zero_int() {
            Ok(EncodedData::EOR(nwords))
        } else if nwords == P::zero_int() {
            Ok(EncodedData::Zero)
        } else {
            let ret = self.read_padded()?;
            Ok(EncodedData::Data(ret))
        }
    }

    fn read_encoded<T>(&mut self) -> Result<&'a T, ErrorCode<P>> {
        match self.read_encoded_data()? {
            EncodedData::EOR(n) => Err(ErrorCode::UnexpectedEOR(n)),
            EncodedData::Zero => Err(ErrorCode::UnexpectedEOR(P::zero_int())),
            EncodedData::Data(d) => Ok(d),
        }
    }

    #[allow(dead_code)]
    fn read_encoded_byte_slice(&mut self) -> Result<&'a [u8], ErrorCode<P>> {
        match self.read_encoded_data_byte_slice()? {
            EncodedData::EOR(n) => Err(ErrorCode::UnexpectedEOR(n)),
            EncodedData::Zero => Err(ErrorCode::UnexpectedEOR(P::zero_int())),
            EncodedData::Data(d) => Ok(d),
        }
    }

    fn read_encoded_slice<T>(&mut self) -> Result<&'a [T], ErrorCode<P>> {
        match self.read_encoded_data_slice()? {
            EncodedData::EOR(n) => Err(ErrorCode::UnexpectedEOR(n)),
            EncodedData::Zero => Err(ErrorCode::UnexpectedEOR(P::zero_int())),
            EncodedData::Data(d) => Ok(d),
        }
    }

    fn read_encoded_value<T: PartialEq + fmt::Debug>(
        &mut self,
        expected_value: &T,
    ) -> Result<&'a T, ErrorCode<P>> {
        let value = self.read_encoded()?;
        if value != expected_value {
            //eprintln!("{:?} != {:?}", value, expected_value);
            return Err(ErrorCode::UnexpectedValue);
        }
        Ok(value)
    }

    fn read_header(&mut self) -> Result<FileHeader<P>, ErrorCode<P>> {
        let date: Date<P> = *self.read_encoded()?;
        let _ = self.read_encoded_value(&P::header_code())?;
        let label = *self.read_encoded()?;
        let _ = self.read_padded_value(&P::Int::from(-1))?;
        let _ = self.read_padded_value(&P::Int::from(0))?;
        Ok(FileHeader { date, label })
    }

    fn read_table_record(&mut self) -> Result<Option<&'a [u8]>, ErrorCode<P>> {
        self.read_encoded_value(&P::Int::from(0))?;
        match self.read_encoded_data_slice()? {
            EncodedData::Data(data) => {
                let record_num: &P::Int = self.read_padded()?;
                if record_num >= &P::zero_int() {
                    return Err(ErrorCode::ExpectedEOR(*record_num));
                }
                Ok(Some(data))
            }
            EncodedData::Zero => Ok(None),
            EncodedData::EOR(n) => Err(ErrorCode::UnexpectedEOR(n)),
        }
    }

    fn read_datablock(&mut self) -> Result<Option<DataBlock<'a, P>>, ErrorCode<P>> {
        let name = match self.read_encoded_data()? {
            EncodedData::EOR(n) => return Err(ErrorCode::UnexpectedEOR(n)),
            EncodedData::Zero => return Ok(None),
            EncodedData::Data(name) => *name,
        };
        self.read_padded_value(&P::Int::from(-1))?;
        let trailer = *self.read_encoded()?;
        self.read_padded_value(&P::Int::from(-2))?;
        let record_type = DataBlockType::parse(*self.read_encoded::<P::Int>()?)?;
        let header = self.read_encoded_slice()?;
        self.read_padded_value(&P::Int::from(-3))?;
        let mut records = vec![];
        while let Some(record) = self.read_table_record()? {
            records.push(record);
        }
        Ok(Some(DataBlock {
            name,
            trailer,
            record_type,
            header,
            records,
        }))
    }

    fn inner_parse(&mut self) -> Result<OP2<'a, P>, ErrorCode<P>> {
        let header = self.read_header()?;
        let mut blocks = vec![];
        while let Some(block) = self.read_datablock()? {
            blocks.push(block);
        }
        if self.buffer.is_empty() {
            Ok(OP2 { header, blocks })
        } else {
            Err(ErrorCode::BytesRemaining)
        }
    }

    fn parse(&mut self) -> std::result::Result<OP2<'a, P>, Error<'a, P>> {
        self.inner_parse().map_err(|code| Error {
            code,
            remaining: self.buffer,
        })
    }
}

pub fn parse_buffer_single(buffer: &[u8]) -> Result<OP2<SinglePrecision>, Error<SinglePrecision>> {
    let mut parser = OP2Parser {
        buffer,
        precision: std::marker::PhantomData,
    };
    parser.parse()
}

pub fn parse_buffer_double(buffer: &[u8]) -> Result<OP2<DoublePrecision>, Error<DoublePrecision>> {
    let mut parser = OP2Parser {
        buffer,
        precision: std::marker::PhantomData,
    };
    parser.parse()
}

#[test]
fn test_parse_buffer() {
    let buf = std::fs::read("tests/op2test32.op2").unwrap();
    let op2 = match parse_buffer_single(&buf) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{}", e);
            assert!(false);
            return;
        }
    };
    assert_eq!(
        op2.header.date,
        Date::<_> {
            month: 8,
            day: 13,
            year: 18
        }
    );
    assert_eq!(
        op2.header.label,
        [SingleWord(*b"NX11"), SingleWord(*b".0.2")]
    );
    assert_eq!(
        op2.blocks[0].name,
        [SingleWord(*b"PVT0"), SingleWord(*b"    ")]
    );
    assert_eq!(op2.blocks[0].trailer, [101, 13, 0, 0, 0, 0, 0]);
    assert_eq!(op2.blocks[0].record_type, DataBlockType::Table);
    assert_eq!(
        op2.blocks[0].header,
        [SingleWord(*b"PVT "), SingleWord(*b"    ")]
    );
    assert_eq!(op2.blocks[0].records.len(), 1);
}

#[test]
fn test_parse_buffer_64() {
    let buf = std::fs::read("tests/op2test64.op2").unwrap();
    let op2 = match parse_buffer_double(&buf) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{}", e);
            assert!(false);
            return;
        }
    };
    assert_eq!(
        op2.header.date,
        Date::<_> {
            month: 7,
            day: 10,
            year: 18
        }
    );
    //assert_eq!(op2.header.label, *b"NX11.0.2");
    //assert_eq!(op2.blocks[0].name, *b"PVT0    ");
    assert_eq!(op2.blocks[0].trailer, [101, 13, 0, 0, 0, 0, 0]);
    assert_eq!(op2.blocks[0].record_type, DataBlockType::Table);
    assert_eq!(
        op2.blocks[0].header,
        [
            DoubleWord(*b"PVT ", *b"    "),
            DoubleWord(*b"    ", *b"    ")
        ]
    );
    assert_eq!(op2.blocks[0].records.len(), 1);
}
