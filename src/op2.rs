use std::mem;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ErrorCode {
    #[error("Bytes remaining")]
    BytesRemaining,
    #[error("Unexpected EOF")]
    UnexpectedEOF,
    #[error("Unexpected EOR ({0})")]
    UnexpectedEOR(i32),
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
    UnknownDataBlockType(i32),
    #[error("ExpectedEOR found {0}")]
    ExpectedEOR(i32),
    #[error("Expected EOR found {0:?}")]
    ExpectedEOR2(EncodedSize),
    #[error("ExpectedData")]
    ExpectedData,
}

#[derive(Debug, Error)]
#[error("{0}\nnext bytes:\n{1:?}",code,&remaining[..std::cmp::min(remaining.len(),20)])]
pub struct Error<'a> {
    code: ErrorCode,
    remaining: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    month: i32,
    day: i32,
    year: i32,
}

#[derive(Debug, PartialEq)]
pub struct FileHeader {
    date: Date,
    label: [u8; 8], // Might want to make this fixed length at some point
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataBlockType {
    Table,
    Matrix,
    StringFactor,
    MatrixFactor,
}

impl std::convert::TryFrom<&i32> for DataBlockType {
    type Error = ErrorCode;
    fn try_from(v: &i32) -> Result<Self, Self::Error> {
        match *v {
            0 => Ok(DataBlockType::Table),
            1 => Ok(DataBlockType::Matrix),
            2 => Ok(DataBlockType::StringFactor),
            3 => Ok(DataBlockType::MatrixFactor),
            n => Err(ErrorCode::UnknownDataBlockType(n)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DataBlock<'a> {
    name: [u8; 8],
    trailer: [i32; 7],
    record_type: DataBlockType,
    header: &'a [u8],
    records: Vec<&'a [u8]>,
}

#[derive(Debug, PartialEq)]
pub struct OP2<'a> {
    header: FileHeader,
    blocks: Vec<DataBlock<'a>>,
}

#[derive(Debug)]
pub enum EncodedSize {
    Data(i32),
    Zero,
    EOR(i32),
}

pub enum EncodedData<T> {
    Data(T),
    Zero,
    EOR(i32),
}

struct OP2Parser<'a> {
    buffer: &'a [u8],
}

impl<'a> OP2Parser<'a> {
    #[inline]
    fn take(&mut self, n: usize) -> Result<&'a [u8], ErrorCode> {
        if self.buffer.len() < n {
            return Err(ErrorCode::UnexpectedEOF);
        }
        let (ret, buffer) = self.buffer.split_at(n);
        self.buffer = buffer;
        Ok(ret)
    }

    fn read_i32(&mut self) -> Result<i32, ErrorCode> {
        use std::convert::TryInto;
        let sl = self.take(4)?;
        Ok(i32::from_le_bytes(sl.try_into().unwrap()))
    }

    fn read_i32_value(&mut self, expected: i32) -> Result<(), ErrorCode> {
        let found = self.read_i32()?;
        if found != expected {
            return Err(ErrorCode::UnexpectedDataSize(expected, found));
        }
        Ok(())
    }

    fn read_padded<T>(&mut self) -> Result<&'a T, ErrorCode> {
        let expected = mem::size_of::<T>();
        if expected > i32::MAX as usize {
            return Err(ErrorCode::ReadTooLarge);
        }
        if expected > i32::MAX as usize {
            return Err(ErrorCode::ReadTooLarge);
        }
        self.read_i32_value(expected as i32)?;
        let res = self.take(expected)?;
        self.read_i32_value(expected as i32)?;
        let (begin, res, end) = unsafe { res.align_to::<T>() };
        if !begin.is_empty() {
            return Err(ErrorCode::AlignmentError);
        }
        #[cfg(debug_assertions)]
        if !end.is_empty() {
            return Err(ErrorCode::BytesRemaining);
        }
        return Ok(&res[0]);
    }

    fn read_padded_value<T: PartialEq>(&mut self, expected_value: &T) -> Result<&'a T, ErrorCode> {
        let value = self.read_padded()?;
        if value != expected_value {
            return Err(ErrorCode::UnexpectedValue);
        }
        return Ok(value);
    }

    fn read_padded_slice(&mut self) -> Result<&'a [u8], ErrorCode> {
        let n = self.read_i32()?;
        if n < 1 {
            return Err(ErrorCode::NegativeRead(n));
        }
        let res = self.take(n as usize)?;
        if n != self.read_i32()? {
            return Err(ErrorCode::UnexpectedValue);
        }
        return Ok(res);
    }

    fn read_encoded_slice(&mut self) -> Result<EncodedData<&'a [u8]>, ErrorCode> {
        let nwords: i32 = *self.read_padded()?;
        if nwords < 0 {
            Ok(EncodedData::EOR(nwords))
        } else if nwords == 0 {
            Ok(EncodedData::Zero)
        } else {
            let nbytes = (nwords as usize) * 4;
            let ret = self.read_padded_slice()?;
            if ret.len() != nbytes {
                return Err(ErrorCode::UnexpectedDataLength(nbytes, ret.len()));
            }
            Ok(EncodedData::Data(ret))
        }
    }

    fn read_encoded<T>(&mut self) -> Result<EncodedData<&'a T>, ErrorCode> {
        let nwords: i32 = *self.read_padded()?;
        if nwords < 0 {
            Ok(EncodedData::EOR(nwords))
        } else if nwords == 0 {
            Ok(EncodedData::Zero)
        } else {
            let ret = self.read_padded()?;
            Ok(EncodedData::Data(ret))
        }
    }

    fn read_encoded_data<T>(&mut self) -> Result<&'a T, ErrorCode> {
        match self.read_encoded()? {
            EncodedData::EOR(n) => Err(ErrorCode::UnexpectedEOR(n)),
            EncodedData::Zero => Err(ErrorCode::UnexpectedEOR(0)),
            EncodedData::Data(d) => Ok(d),
        }
    }

    fn read_encoded_data_slice(&mut self) -> Result<&'a [u8], ErrorCode> {
        match self.read_encoded_slice()? {
            EncodedData::EOR(n) => Err(ErrorCode::UnexpectedEOR(n)),
            EncodedData::Zero => Err(ErrorCode::UnexpectedEOR(0)),
            EncodedData::Data(d) => Ok(d),
        }
    }

    fn read_encoded_value<T: PartialEq>(&mut self, expected_value: &T) -> Result<&'a T, ErrorCode> {
        let value = self.read_encoded_data()?;
        if value != expected_value {
            return Err(ErrorCode::UnexpectedValue);
        }
        return Ok(value);
    }

    fn read_header(&mut self) -> Result<FileHeader, ErrorCode> {
        let date: Date = *self.read_encoded_data()?;
        let _ = self.read_encoded_value(b"NASTRAN FORT TAPE ID CODE - ")?;
        let label = *self.read_encoded_data()?;
        let _ = self.read_padded_value(&-1i32)?;
        let _ = self.read_padded_value(&0i32)?;
        Ok(FileHeader { date, label })
    }

    fn read_table_record(&mut self) -> Result<Option<&'a [u8]>, ErrorCode> {
        self.read_encoded_value(&0i32)?;
        match self.read_encoded_slice()? {
            EncodedData::Data(data) => {
                let record_num: i32 = *self.read_padded()?;
                if record_num >= 0 {
                    return Err(ErrorCode::ExpectedEOR(record_num));
                }
                Ok(Some(data))
            }
            EncodedData::Zero => Ok(None),
            EncodedData::EOR(n) => Err(ErrorCode::UnexpectedEOR(n)),
        }
    }

    fn read_datablock(&mut self) -> Result<Option<DataBlock<'a>>, ErrorCode> {
        use std::convert::TryInto;
        let name = match self.read_encoded()? {
            EncodedData::EOR(n) => return Err(ErrorCode::UnexpectedEOR(n)),
            EncodedData::Zero => return Ok(None),
            EncodedData::Data(name) => *name,
        };
        self.read_padded_value(&-1)?;
        let trailer = *self.read_encoded_data()?;
        self.read_padded_value(&-2)?;
        let record_type = self.read_encoded_data::<i32>()?.try_into()?;
        let header: &[u8] = self.read_encoded_data_slice()?;
        self.read_padded_value(&-3)?;
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

    fn inner_parse(&mut self) -> Result<OP2<'a>, ErrorCode> {
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

    fn parse(&mut self) -> std::result::Result<OP2<'a>, Error<'a>> {
        self.inner_parse().map_err(|code| Error {
            code,
            remaining: self.buffer,
        })
    }
}

pub fn parse_buffer(buffer: &[u8]) -> Result<OP2<'_>, Error<'_>> {
    let mut parser = OP2Parser { buffer };
    parser.parse()
}

#[test]
fn test_parse_buffer() {
    let buf = std::fs::read("tests/op2test32.op2").unwrap();
    let op2 = match parse_buffer(&buf) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{}", e);
            assert!(false);
            return;
        }
    };
    assert_eq!(
        op2.header.date,
        Date {
            month: 8,
            day: 13,
            year: 18
        }
    );
    assert_eq!(op2.header.label, *b"NX11.0.2");
    assert_eq!(op2.blocks[0].name, *b"PVT0    ");
    assert_eq!(op2.blocks[0].trailer, [101, 13, 0, 0, 0, 0, 0]);
    assert_eq!(op2.blocks[0].record_type, DataBlockType::Table);
}
