mod oef;
mod prelude;

use std::fmt;
use std::mem;
use std::path::Path;

use bstr::ByteSlice;
use thiserror::Error;
use tracing::{span, trace, Level};

pub struct WordsDebug<'a,W:Word>(&'a [W]);

impl <'a,W:Word> fmt::Debug for WordsDebug<'a,W> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for w in self.0 {
            write!(f, "{}", w.as_single().as_bstr())?;
        }
        Ok(())
    }
}

pub trait WordsDebugExt<'a, W: Word>  {
    fn debug_words(&'a self) -> WordsDebug<'a, W>;
}

impl <'a, W: Word> WordsDebugExt<'a, W> for &'a [W] {
    fn debug_words(&self) -> WordsDebug<W> {
        WordsDebug(self)
    }
}


pub trait Word: fmt::Debug + fmt::Display + PartialEq + Copy {
    fn as_single(&self) -> [u8; 4];
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(C)]
pub struct SingleWord([u8; 4]);

// SAFETY All zeros is a valid value
unsafe impl bytemuck::Zeroable for SingleWord {}
// SAFETY Any value is valid, there is no padding, the underlying type is Pod and its repr(C)
unsafe impl bytemuck::Pod for SingleWord {}

impl fmt::Display for SingleWord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.as_bstr())
    }
}

impl Word for SingleWord {
    fn as_single(&self) -> [u8; 4] {
        self.0
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(C)]
pub struct DoubleWord([u8; 4], [u8; 4]);

// SAFETY All zeros is a valid value
unsafe impl bytemuck::Zeroable for DoubleWord {}
// SAFETY Any value is valid, there is no padding, the underlying type is Pod and its repr(C)
unsafe impl bytemuck::Pod for DoubleWord {}

impl fmt::Display for DoubleWord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // This doesn't output the second value since that is unused in the double width op2
        write!(f, "{}", self.0.as_bstr())
    }
}

impl Word for DoubleWord {
    fn as_single(&self) -> [u8; 4] {
        debug_assert!(&self.1 == b"    ");
        self.0
    }
}

pub trait Precision: fmt::Debug + Sized + Copy + bytemuck::Pod {
    type Int: fmt::Debug
        + fmt::Display
        + PartialEq
        + PartialOrd
        + Copy
        + Into<i64>
        + From<i32>
        + std::ops::Div<Self::Int, Output=Self::Int>
        + bytemuck::Pod;
    type UInt: fmt::Debug + fmt::Display + bytemuck::Pod;
    type Float: fmt::Debug + fmt::Display + bytemuck::Pod;
    type Word: Word + bytemuck::Pod;

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

    fn int_to_i32(v: Self::Int) -> i32 {
        let v: i64 = v.into();
        debug_assert!(v < i32::MAX as i64);
        v as i32
    }

    fn word_to_int(v: Self::Word) -> Self::Int;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SinglePrecision;

// SAFETY Since this is ZST, it holds no data
unsafe impl bytemuck::Zeroable for SinglePrecision {}
// SAFETY Since this is ZST, it holds no data
unsafe impl bytemuck::Pod for SinglePrecision {}

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

    fn word_to_int(word: Self::Word) -> Self::Int {
        i32::from_le_bytes(word.0)
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

// SAFETY Since this is ZST, it holds no data
unsafe impl bytemuck::Zeroable for DoublePrecision {}
// SAFETY Since this is ZST, it holds no data
unsafe impl bytemuck::Pod for DoublePrecision {}

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

    fn word_to_int(word: Self::Word) -> Self::Int {
        let mut temp = [0u8;8];
        temp[..4].copy_from_slice(&word.0);
        temp[4..].copy_from_slice(&word.1);
        i64::from_le_bytes(temp)
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
    #[error("Unaligned Value")]
    UnalignedValue,
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
#[error("{code}")]
pub struct Error<P: Precision> {
    code: ErrorCode<P>,
    remaining: Option<Indexed<MaybeAligned, u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Date<P: Precision> {
    month: P::Int,
    day: P::Int,
    year: P::Int,
}

// SAFETY All zeros is a valid value even if it doesn't have a real meaning
unsafe impl<P: Precision> bytemuck::Zeroable for Date<P> {}
// SAFETY Any value is valid, there is no padding since all values are the same size, the
// underlying type is Pod per requirements of Precision and its repr(C)
unsafe impl<P: Precision> bytemuck::Pod for Date<P> {}

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
pub struct Aligned;
#[derive(Debug, PartialEq)]
pub struct MaybeAligned;
#[derive(Debug, PartialEq)]
pub struct Unaligned;

pub trait Alignment: fmt::Debug {}

impl Alignment for Aligned {}
impl Alignment for MaybeAligned {}
impl Alignment for Unaligned {}

#[derive(Debug, PartialEq)]
pub struct Indexed<A, T> {
    start: usize,
    end: usize,
    alignment: std::marker::PhantomData<A>,
    data: std::marker::PhantomData<T>,
}

impl<A, T> Indexed<A, T>
where
    A: Alignment,
    T: bytemuck::Pod,
{
    fn new(start: usize, end: usize) -> Self {
        Indexed {
            start,
            end,
            alignment: std::marker::PhantomData,
            data: std::marker::PhantomData,
        }
    }
}

impl<T> Indexed<Aligned, T>
where
    T: bytemuck::Pod,
{
    pub fn read<'b>(&self, file_buffer: &'b [u8]) -> &'b T {
        let buf = &file_buffer[self.start..self.end];
        bytemuck::from_bytes(buf)
    }

    pub fn read_value(&self, file_buffer: &[u8]) -> T {
        *self.read(file_buffer)
    }
}

impl<T> Indexed<Unaligned, T>
where
    T: bytemuck::Pod,
{
    pub fn read_value(&self, file_buffer: &[u8]) -> T {
        let buf = &file_buffer[self.start..self.end];
        debug_assert!(buf.len() == std::mem::size_of::<T>());
        // SAFETY this slice is the same size the type. And this slice would
        // only have been saved if it was valid to read it
        unsafe { std::ptr::read_unaligned(buf.as_ptr() as *const T) }
    }
}

impl<T> Indexed<MaybeAligned, T>
where
    T: bytemuck::Pod,
{
    pub fn try_read<'b>(&self, file_buffer: &'b [u8]) -> Option<&'b T> {
        let buf = &file_buffer[self.start..self.end];
        bytemuck::try_from_bytes(buf).ok()
    }

    fn read_value(&self, file_buffer: &[u8]) -> T {
        let buf = &file_buffer[self.start..self.end];
        if (buf.as_ptr() as usize) % std::mem::size_of::<T>() == 0 {
            *bytemuck::from_bytes(buf)
        } else {
            debug_assert!(buf.len() == std::mem::size_of::<T>());
            // SAFETY this slice is the same size the type. And this slice would
            // only have been saved if it was valid to read it
            unsafe { std::ptr::read_unaligned(buf.as_ptr() as *const T) }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct IndexedSlice<A, T> {
    start: usize,
    end: usize,
    alignment: std::marker::PhantomData<A>,
    data: std::marker::PhantomData<T>,
}

impl<A, T> IndexedSlice<A, T>
where
    A: Alignment,
    T: bytemuck::Pod,
{
    fn new(start: usize, end: usize) -> Self {
        debug_assert!((end - start) % std::mem::size_of::<T>() == 0);
        IndexedSlice {
            start,
            end,
            alignment: std::marker::PhantomData,
            data: std::marker::PhantomData,
        }
    }

    fn len(&self) -> usize {
        (self.end - self.start) / std::mem::size_of::<T>()
    }
}

impl<T> IndexedSlice<Aligned, T>
where
    T: bytemuck::Pod,
{
    pub fn read<'b>(&self, file_buffer: &'b [u8]) -> &'b [T] {
        let buf = &file_buffer[self.start..self.end];
        bytemuck::cast_slice(buf)
    }

    pub fn read_value(&self, file_buffer: &[u8]) -> Vec<T> {
        self.read(file_buffer).to_vec()
    }
}

impl<T> IndexedSlice<Unaligned, T>
where
    T: bytemuck::Pod,
{
    pub fn read_value(&self, file_buffer: &[u8]) -> Vec<T> {
        let buf = &file_buffer[self.start..self.end];
        let mut ret = Vec::with_capacity(self.len());
        let mut offset = 0;
        for _ in 0..self.len() {
            // SAFETY this slice is the same size the type. And this slice would
            // only have been saved if it was valid to read it
            ret.push(unsafe { std::ptr::read_unaligned((&buf[offset..]).as_ptr() as *const T) });
            offset += std::mem::size_of::<T>();
        }
        ret
    }
}

impl<T> IndexedSlice<MaybeAligned, T>
where
    T: bytemuck::Pod,
{
    pub fn try_read<'b>(&self, file_buffer: &'b [u8]) -> Option<&'b [T]> {
        let buf = &file_buffer[self.start..self.end];
        bytemuck::try_cast_slice(buf).ok()
    }

    pub fn read_value(&self, file_buffer: &[u8]) -> Vec<T> {
        let buf = &file_buffer[self.start..self.end];
        if (buf.as_ptr() as usize) % std::mem::size_of::<T>() == 0 {
            bytemuck::cast_slice(buf).to_vec()
        } else {
            let mut ret = Vec::with_capacity(self.len());
            let mut offset = 0;
            for _ in 0..self.len() {
                // SAFETY this slice is the same size the type. And this slice would
                // only have been saved if it was valid to read it
                ret.push(unsafe {
                    std::ptr::read_unaligned((&buf[offset..]).as_ptr() as *const T)
                });
                offset += std::mem::size_of::<T>();
            }
            ret
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct IndexedSlices<A, T> {
    slices: Vec<IndexedSlice<A, T>>,
}

#[derive(Debug, PartialEq)]
pub struct DataBlock<A: Alignment, P: Precision> {
    name: [P::Word; 2],
    trailer: [P::Int; 7],
    record_type: DataBlockType,
    header: IndexedSlice<A, P::Word>,
    records: Vec<Vec<IndexedSlice<Aligned, u8>>>,
}

impl<A: Alignment, P: Precision> DataBlock<A, P> {
    fn name(&self) -> [u8; 8] {
        let mut name = [0; 8];
        name[..4].copy_from_slice(&self.name[0].as_single());
        name[4..8].copy_from_slice(&self.name[1].as_single());
        name
    }
}

#[derive(Debug, PartialEq)]
pub struct OP2MetaData<A: Alignment, P: Precision> {
    header: FileHeader<P>,
    blocks: Vec<DataBlock<A, P>>,
}

impl<A: Alignment, P: Precision> OP2MetaData<A, P> {}

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
    index: usize,
    buffer: &'a [u8],
    precision: std::marker::PhantomData<P>,
}

impl<'a, P> fmt::Debug for OP2Parser<'a, P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("OP2Parser")
            .field("index", &self.index)
            .field("len(buffer)", &self.buffer.len())
            .finish()
    }
}

impl<'a, P: 'a> OP2Parser<'a, P>
where
    P: Precision,
{
    #[inline]
    fn rem(&self) -> usize {
        self.buffer.len() - self.index
    }

    #[inline]
    fn eof(&self) -> bool {
        self.rem() == 0
    }

    #[inline]
    fn take(&mut self, n: usize) -> Result<IndexedSlice<Aligned, u8>, ErrorCode<P>> {
        let span = span!(Level::TRACE, "take", index = self.index);
        let _s = span.enter();

        trace!("n = {}", n);
        if self.rem() < n {
            return Err(ErrorCode::UnexpectedEOF);
        }
        let ret = IndexedSlice::new(self.index, self.index + n);
        self.index += n;
        Ok(ret)
    }

    fn read<T: bytemuck::Pod>(&mut self) -> Result<Indexed<MaybeAligned, T>, ErrorCode<P>> {
        let n = std::mem::size_of::<T>();
        if self.rem() < n {
            return Err(ErrorCode::UnexpectedEOF);
        }
        let ret = Indexed::new(self.index, self.index + n);
        self.index += n;
        Ok(ret)
    }

    fn read_slice<T: bytemuck::Pod>(
        &mut self,
        n_bytes: usize,
    ) -> Result<IndexedSlice<MaybeAligned, T>, ErrorCode<P>> {
        let span = span!(Level::TRACE, "read_slice", index = self.index);
        let _s = span.enter();

        trace!("n_bytes = {}", n_bytes);
        if self.rem() < n_bytes {
            return Err(ErrorCode::UnexpectedEOF);
        }
        let n = std::mem::size_of::<T>();
        if n_bytes % n != 0 {
            return Err(ErrorCode::AlignmentError);
        }
        let buf = &self.buffer[self.index..];
        if (buf.as_ptr() as usize) % std::mem::align_of::<T>() != 0 {
            return Err(ErrorCode::UnalignedValue);
        }
        let ret = IndexedSlice::new(self.index, self.index + n_bytes);
        self.index += n_bytes;
        Ok(ret)
    }

    fn read_byte_slice(
        &mut self,
        n_bytes: usize,
    ) -> Result<IndexedSlice<Aligned, u8>, ErrorCode<P>> {
        let span = span!(Level::TRACE, "read_byte_slice", index = self.index);
        let _s = span.enter();

        trace!("n_bytes = {}", n_bytes);

        if self.rem() < n_bytes {
            return Err(ErrorCode::UnexpectedEOF);
        }
        let ret = IndexedSlice::new(self.index, self.index + n_bytes);
        self.index += n_bytes;
        Ok(ret)
    }

    fn read_i32(&mut self) -> Result<i32, ErrorCode<P>> {
        let span = span!(Level::TRACE, "read_i32", index = self.index);
        let _s = span.enter();

        let mut buf = [0u8; 4];
        let sl = &self.take(4)?.read(self.buffer);
        buf.copy_from_slice(sl);
        Ok(i32::from_le_bytes(buf))
    }

    fn read_i32_expected(&mut self, expected: i32) -> Result<(), ErrorCode<P>> {
        let span = span!(Level::TRACE, "read_i32_expected", index = self.index);
        let _s = span.enter();
        trace!("expected = {}", expected);
        let found = self.read_i32()?;
        if found != expected {
            return Err(ErrorCode::UnexpectedDataSize(expected, found));
        }
        Ok(())
    }

    fn read_padded<T: bytemuck::Pod>(&mut self) -> Result<Indexed<MaybeAligned, T>, ErrorCode<P>> {
        let span = span!(Level::TRACE, "read_padded", index = self.index);
        let _s = span.enter();
        let expected = mem::size_of::<T>();
        let expected_i = P::i32_from_usize(expected)?;
        self.read_i32_expected(expected_i)?;
        let res = self.read()?;
        self.read_i32_expected(expected_i)?;
        Ok(res)
    }

    fn read_padded_expected<T: PartialEq + fmt::Debug + bytemuck::Pod>(
        &mut self,
        expected_value: &T,
    ) -> Result<(), ErrorCode<P>> {
        let span = span!(Level::TRACE, "read_padded_expected", index = self.index);
        let _s = span.enter();
        trace!("expected_value = {:?}", expected_value);
        let v = self.read_padded::<T>()?;
        // This could potentially be optimized since we really only need to compare the
        // underlying bytes, so the concerns about alignment are not necessary
        if &v.read_value(self.buffer) != expected_value {
            return Err(ErrorCode::UnexpectedValue);
        }
        Ok(())
    }

    fn read_padded_slice<T: bytemuck::Pod>(
        &mut self,
    ) -> Result<IndexedSlice<MaybeAligned, T>, ErrorCode<P>> {
        let span = span!(Level::TRACE, "read_padded_slice", index = self.index);
        let _s = span.enter();

        let size = std::mem::size_of::<T>();
        let n = self.read_i32()?;
        if n < 1 {
            return Err(ErrorCode::NegativeRead(n));
        }
        if n as usize % size != 0 {
            return Err(ErrorCode::AlignmentError);
        }
        let res = self.read_slice(n as usize)?;
        let expected = n;
        let n = self.read_i32()?;
        if n != expected {
            return Err(ErrorCode::UnexpectedValue);
        }
        Ok(res)
    }

    fn read_padded_byte_slice(&mut self) -> Result<IndexedSlice<Aligned, u8>, ErrorCode<P>> {
        let span = span!(Level::TRACE, "read_padded_byte_slice", index = self.index);
        let _s = span.enter();

        let n = self.read_i32()?;
        trace!("n = {}", n);
        if n < 1 {
            return Err(ErrorCode::NegativeRead(n));
        }
        let res = self.read_byte_slice(n as usize)?;
        let expected = n;
        let n = self.read_i32()?;
        if n != expected {
            return Err(ErrorCode::UnexpectedValue);
        }
        Ok(res)
    }

    fn read_encoded_data_slice<T: bytemuck::Pod>(
        &mut self,
    ) -> Result<EncodedData<P, IndexedSlice<MaybeAligned, T>>, ErrorCode<P>> {
        let span = span!(Level::TRACE, "read_encoded_data_slice", index = self.index);
        let _s = span.enter();

        let nwords: P::Int = self.read_padded()?.read_value(self.buffer);
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

    fn read_encoded_byte_slice(
        &mut self,
    ) -> Result<EncodedData<P, IndexedSlice<Aligned, u8>>, ErrorCode<P>> {
        let span = span!(Level::TRACE, "read_encoded_byte_slice", index = self.index);
        let _s = span.enter();

        let nwords: P::Int = self.read_padded()?.read_value(self.buffer);
        trace!("nwords = {}", nwords);
        if nwords < P::zero_int() {
            Ok(EncodedData::EOR(nwords))
        } else if nwords == P::zero_int() {
            Ok(EncodedData::Zero)
        } else {
            let nwords: i64 = nwords.into();
            let nbytes = (nwords as usize) * P::WORDSIZE;
            let ret = self.read_padded_byte_slice()?;
            if ret.len() != nbytes {
                return Err(ErrorCode::UnexpectedDataLength(nbytes, ret.len()));
            }
            Ok(EncodedData::Data(ret))
        }
    }

    fn read_encoded_data<T: bytemuck::Pod>(
        &mut self,
    ) -> Result<EncodedData<P, Indexed<MaybeAligned, T>>, ErrorCode<P>> {
        let span = span!(Level::TRACE, "read_encoded_data", index = self.index);
        let _s = span.enter();

        let nwords: P::Int = self.read_padded()?.read_value(self.buffer);
        if nwords < P::zero_int() {
            Ok(EncodedData::EOR(nwords))
        } else if nwords == P::zero_int() {
            Ok(EncodedData::Zero)
        } else {
            let ret = self.read_padded()?;
            Ok(EncodedData::Data(ret))
        }
    }

    fn read_encoded<T: bytemuck::Pod>(&mut self) -> Result<Indexed<MaybeAligned, T>, ErrorCode<P>> {
        let span = span!(Level::TRACE, "read_encoded", index = self.index);
        let _s = span.enter();

        match self.read_encoded_data()? {
            EncodedData::EOR(n) => Err(ErrorCode::UnexpectedEOR(n)),
            EncodedData::Zero => Err(ErrorCode::UnexpectedEOR(P::zero_int())),
            EncodedData::Data(d) => Ok(d),
        }
    }

    fn read_encoded_slice<T: bytemuck::Pod>(
        &mut self,
    ) -> Result<IndexedSlice<MaybeAligned, T>, ErrorCode<P>> {
        match self.read_encoded_data_slice()? {
            EncodedData::EOR(n) => Err(ErrorCode::UnexpectedEOR(n)),
            EncodedData::Zero => Err(ErrorCode::UnexpectedEOR(P::zero_int())),
            EncodedData::Data(d) => Ok(d),
        }
    }

    fn read_encoded_expected<T: PartialEq + fmt::Debug + bytemuck::Pod>(
        &mut self,
        expected_value: &T,
    ) -> Result<(), ErrorCode<P>> {
        let value = self.read_encoded::<T>()?;
        // This could potentially be optimized since we really only need to compare the
        // underlying bytes, so the concerns about alignment are not necessary
        if expected_value != &value.read_value(self.buffer) {
            return Err(ErrorCode::UnexpectedValue);
        }
        Ok(())
    }

    fn read_header(&mut self) -> Result<FileHeader<P>, ErrorCode<P>> {
        let date: Date<P> = self.read_encoded()?.read_value(self.buffer);
        self.read_encoded_expected(&P::header_code())?;
        let label = self.read_encoded()?.read_value(self.buffer);
        self.read_padded_expected(&P::Int::from(-1))?;
        self.read_padded_expected(&P::Int::from(0))?;
        Ok(FileHeader { date, label })
    }

    fn read_table_record(
        &mut self,
    ) -> Result<Option<Vec<IndexedSlice<Aligned, u8>>>, ErrorCode<P>> {
        let span = span!(Level::TRACE, "read_table_record", index = self.index);
        let _s = span.enter();

        self.read_encoded_expected(&P::Int::from(0))?;
        let mut res = Vec::new();
        match self.read_encoded_byte_slice()? {
            EncodedData::Data(data) => {
                trace!("found data");
                res.push(data);
                loop {
                    let record_num: P::Int = self.read_padded()?.read_value(self.buffer);
                    // 65538 is the max record size
                    if record_num >= P::zero_int() {
                        let data = self.read_padded_byte_slice()?;
                        res.push(data);
                    } else {
                        break Ok(Some(res));
                    }
                }
            }
            EncodedData::Zero => Ok(None),
            EncodedData::EOR(n) => Err(ErrorCode::UnexpectedEOR(n)),
        }
    }

    fn read_datablock(&mut self) -> Result<Option<DataBlock<MaybeAligned, P>>, ErrorCode<P>> {
        let span = span!(Level::TRACE, "read_datablock", index = self.index);
        let _s = span.enter();

        if self.eof() {
            return Ok(None);
        }

        let name = match self.read_encoded_data()? {
            EncodedData::EOR(n) => return Err(ErrorCode::UnexpectedEOR(n)),
            EncodedData::Zero => return Ok(None),
            EncodedData::Data(name) => name.read_value(self.buffer),
        };
        self.read_padded_expected(&P::Int::from(-1))?;
        let trailer = self.read_encoded()?.read_value(self.buffer);
        self.read_padded_expected(&P::Int::from(-2))?;
        let record_type =
            DataBlockType::parse(self.read_encoded::<P::Int>()?.read_value(self.buffer))?;
        let header = self.read_encoded_slice()?;
        self.read_padded_expected(&P::Int::from(-3))?;
        let mut records = vec![];
        while let Some(r) = self.read_table_record()? {
            records.push(r);
        }
        Ok(Some(DataBlock {
            name,
            trailer,
            record_type,
            header,
            records,
        }))
    }

    fn inner_parse(&mut self) -> Result<OP2MetaData<MaybeAligned, P>, ErrorCode<P>> {
        let span = span!(Level::TRACE, "inner_parse", index = self.index);
        let _s = span.enter();

        let header = self.read_header()?;
        let mut blocks = vec![];
        while let Some(block) = self.read_datablock()? {
            blocks.push(block);
        }
        if self.rem() == 0 {
            Ok(OP2MetaData { header, blocks })
        } else {
            Err(ErrorCode::BytesRemaining)
        }
    }

    fn parse(&mut self) -> std::result::Result<OP2MetaData<MaybeAligned, P>, Error<P>> {
        self.inner_parse().map_err(|code| Error {
            code,
            remaining: Some(Indexed::new(self.index, self.buffer.len())),
        })
    }
}

pub fn parse_buffer<P: Precision>(buffer: &[u8]) -> Result<OP2MetaData<MaybeAligned, P>, Error<P>> {
    let mut parser = OP2Parser {
        index: 0,
        buffer,
        precision: std::marker::PhantomData,
    };
    parser.parse()
}

// TODO There should be a way to guarantee alignment here, but not sure if that
// requires specialization (or duplicating a bunch of methods)
pub fn parse_buffer_single(
    buffer: &[u8],
) -> Result<OP2MetaData<MaybeAligned, SinglePrecision>, Error<SinglePrecision>> {
    let mut parser = OP2Parser {
        index: 0,
        buffer,
        precision: std::marker::PhantomData,
    };
    parser.parse()
}

pub fn parse_buffer_double(
    buffer: &[u8],
) -> Result<OP2MetaData<MaybeAligned, DoublePrecision>, Error<DoublePrecision>> {
    let mut parser = OP2Parser {
        index: 0,
        buffer,
        precision: std::marker::PhantomData,
    };
    parser.parse()
}

// TODO Look at whether I should have a drop implementation for safety or otherwise
#[derive(Debug)]
struct MmapFile {
    file: std::fs::File,
    mmap: memmap2::Mmap,
}

impl MmapFile {
    fn new(file: std::fs::File) -> std::io::Result<Self> {
        use fs2::FileExt;
        file.lock_exclusive()?;
        // SAFETY: Should be safe since open_file gets an exclusive lock on the whole file and the
        // buffer gets dropped before the end of the function while the file lock is still held
        let mmap = unsafe { memmap2::Mmap::map(&file) }?;
        Ok(Self { file, mmap })
    }

    fn open(filename: impl AsRef<Path>) -> std::io::Result<Self> {
        let file = std::fs::File::open(filename)?;
        Self::new(file)
    }

    fn as_buf(&self) -> &[u8] {
        self.mmap.as_ref()
    }
}

#[derive(Debug)]
pub struct OP2File<A: Alignment, P: Precision> {
    file: MmapFile,
    meta: OP2MetaData<A, P>,
}

impl<A: Alignment, P: Precision> OP2File<A, P> {
    pub fn block_names<'slf>(&'slf self) -> impl Iterator<Item = [u8; 8]> + 'slf {
        self.meta.blocks.iter().map(|b| b.name())
    }

    pub fn oef(&self) -> Option<()> {
        let block = self.meta.blocks.iter().find(|b| &b.name() == b"OEF1X   ")?;
        debug_assert!(block.records.len() % 2 == 0);
        let n = block.records.len() / 2;
        for i in 0..n {
            let ident_slices = &block.records[i * 2];
            debug_assert!(ident_slices.len() == 1);
            let ident = ident_slices[0].read(self.file.as_buf());
            debug_assert!(ident.len() == std::mem::size_of::<oef::Ident<P>>());
            let ident: &oef::Ident<P> = bytemuck::from_bytes(ident);
            //println!("{:?}", ident);
            println!("{:?}", ident.metadata());
            //let data  = block.records[i*2 + 1].read(self.file.as_buf());
        }
        Some(())
    }
}

pub fn parse_file<P: Precision>(
    filename: impl AsRef<Path>,
) -> Result<OP2File<MaybeAligned, P>, Error<P>> {
    let file = MmapFile::open(filename).map_err(|e| Error {
        code: ErrorCode::IO(e),
        remaining: None,
    })?;
    let meta = parse_buffer(file.as_buf())?;
    Ok(OP2File { file, meta })
}

// TODO There should be a way to guarantee alignment here, but not sure if that
// requires specialization (or duplicating a bunch of methods)
pub fn parse_file_single(
    filename: impl AsRef<Path>,
) -> Result<OP2File<MaybeAligned, SinglePrecision>, Error<SinglePrecision>> {
    parse_file(filename.as_ref())
}

pub fn parse_file_double(
    filename: impl AsRef<Path>,
) -> Result<OP2File<MaybeAligned, DoublePrecision>, Error<DoublePrecision>> {
    parse_file(filename.as_ref())
}

#[cfg(test)]
mod test {
    use super::*;

    #[macro_use]
    mod macros {
        #[repr(C)] // guarantee 'bytes' comes after '_align'
        pub struct AlignedAs<Align, Bytes: ?Sized> {
            pub _align: [Align; 0],
            pub bytes: Bytes,
        }

        macro_rules! include_bytes_align_as {
            ($align_ty:ty, $path:literal) => {{
                // const block expression to encapsulate the static
                use self::macros::AlignedAs;

                // this assignment is made possible by CoerceUnsized
                static ALIGNED: &AlignedAs<$align_ty, [u8]> = &AlignedAs {
                    _align: [],
                    bytes: *include_bytes!($path),
                };

                &ALIGNED.bytes
            }};
        }
    }

    #[test]
    fn test_parse_buffer() {
        // include_bytes here is just to work around some issues related
        // to filesystem protections on my machine
        let buf = include_bytes_align_as!(u64, "../tests/op2test32.op2");
        let op2 = match parse_buffer_single(&buf[..]) {
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
        let header = op2.blocks[0].header.read_value(buf);
        assert_eq!(header, vec![SingleWord(*b"PVT "), SingleWord(*b"    ")]);
        assert_eq!(op2.blocks[0].records.len(), 1);
    }

    #[test]
    fn test_parse_buffer_64() {
        // include_bytes here is just to work around some issues related
        // to filesystem protections on my machine
        let buf = include_bytes_align_as!(u64, "../tests/op2test64.op2");
        let op2 = match parse_buffer_double(&buf[..]) {
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
        assert_eq!(
            op2.header.label,
            [
                DoubleWord(*b"NX11", *b"    "),
                DoubleWord(*b".0.2", *b"    ")
            ]
        );
        assert_eq!(
            op2.blocks[0].name,
            [
                DoubleWord(*b"PVT0", *b"    "),
                DoubleWord(*b"    ", *b"    ")
            ]
        );
        assert_eq!(op2.blocks[0].trailer, [101, 13, 0, 0, 0, 0, 0]);
        assert_eq!(op2.blocks[0].record_type, DataBlockType::Table);
        let header = op2.blocks[0].header.read_value(buf);
        assert_eq!(
            header,
            vec![
                DoubleWord(*b"PVT ", *b"    "),
                DoubleWord(*b"    ", *b"    ")
            ]
        );
        assert_eq!(op2.blocks[0].records.len(), 1);
    }
}

pub enum OneOrTwo {
    One,
    Two
}

pub fn fun1<P: Precision>(value: P::Int) -> OneOrTwo {
    let value: i64 = value.into();
    if matches!(value/1000, 2|3|6) {
        OneOrTwo::Two
    } else {
        OneOrTwo::One
    }
}

pub fn fun2<P: Precision>(value: P::Int) -> i32 {
    let value: i64 = value.into();
    (value % 100) as i32
}

pub fn fun3<P: Precision>(value: P::Int) -> i32 {
    let value: i64 = value.into();
    (value % 1000) as i32
}

pub fn fun4<P: Precision>(value: P::Int) -> P::Int {
    value / P::Int::from(10)
}

pub fn fun5<P: Precision>(value: P::Int) -> i32 {
    let value: i64 = value.into();
    (value % 10) as i32
}

pub fn fun6<P: Precision>(value: P::Int) -> P::Int {
    unimplemented!()
}

pub fn fun7<P: Precision>(value: P::Int) -> i32 {
    let value: i64 = value.into();
    match value/1000 {
        0|2 => 0,
        1|3 => 1,
        _ => 2
    }
}
