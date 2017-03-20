use std::mem::{size_of, transmute};
use std::slice::from_raw_parts;
use std::marker::Sized;
use std::borrow::Cow;
use std::fmt;

use nom::{IResult, Needed, be_i64, le_i64, le_i32, le_i8, ErrorKind};

use geom1;

#[derive(Debug)]
struct Date {
    month: i32,
    day: i32,
    year: i32,
}

#[derive(Debug)]
pub struct FileHeader<'a> {
    date: &'a Date,
    label: &'a [u8], // Might want to make this fixed length at some point
}

#[derive(Debug)]
pub enum DataBlockType {
    Table,
    Matrix,
    StringFactor,
    MatrixFactor,
}

pub type DataBlockTrailer<'a> = &'a [i32; 7];

pub enum DataBlock<'a> {
    Generic(GenericDataBlock<'a>),
    OUG(OUG<'a>),
    GEOM1(geom1::DataBlock<'a>),
}

pub struct OP2<'a> {
    pub header: FileHeader<'a>,
    pub blocks: Vec<DataBlock<'a>>,
}

named!(pub read_fortran_data,
       do_parse!(
  length: le_i32 >>
  data: take!(length) >>
  tag!(unsafe { transmute::<i32,[u8;4]>(length)}) >>
  (data)
));

fn i32_to_bytearray(num: i32) -> [u8; 4] {
    unsafe { transmute(num.to_le()) }
}

pub fn read_known_i32(input: &[u8], v: i32) -> IResult<&[u8], ()> {
    tag!(input, i32_to_bytearray(v)).map(|v| ())
}

named!(pub read_fortran_i32<i32>,
  do_parse!(
  apply!(read_known_i32,4) >>
  v: le_i32 >>
  apply!(read_known_i32,4) >>
  (v)
  ));

fn read_fortran_known_i32(input: &[u8], v: i32) -> IResult<&[u8], ()> {
    do_parse!(input,
  apply!(read_known_i32,4) >>
  apply!(read_known_i32,v) >>
  apply!(read_known_i32,4) >>
  ()
  )
}

named!(pub read_nastran_i32<i32>,
  do_parse!(
  apply!(read_fortran_known_i32,1) >>
  value: read_fortran_i32 >>
  (value)
  )
);

pub fn read_nastran_known_i32(input: &[u8], v: i32) -> IResult<&[u8], ()> {
    do_parse!(input,
  apply!(read_fortran_known_i32,1) >>
  apply!(read_fortran_known_i32,v) >>
  ()
  )
}

const WORD_SIZE: i32 = 4;

pub fn read_nastran_tag<'a>(input: &'a [u8], v: &[u8]) -> IResult<&'a [u8], ()> {
    let l: i32 = v.len() as i32;
    do_parse!(input,
  apply!(read_fortran_known_i32,l/WORD_SIZE) >>
  apply!(read_known_i32,l) >>
  tag!(v) >>
  apply!(read_known_i32,l) >>
  ()
  )
}

pub fn read_nastran_data_known_length(input: &[u8], v: i32) -> IResult<&[u8], &[u8]> {
    do_parse!(input,
  apply!(read_fortran_known_i32,v) >>
  apply!(read_known_i32,v*WORD_SIZE) >>
  data: take!(v*WORD_SIZE) >>
  apply!(read_known_i32,v*WORD_SIZE) >>
  (data)
  )
}

pub fn read_nastran_data(input: &[u8]) -> IResult<&[u8], &[u8]> {
    do_parse!(input,
  length: read_fortran_i32 >>
  apply!(read_known_i32,length*WORD_SIZE) >>
  data: take!(length*WORD_SIZE) >>
  apply!(read_known_i32,length*WORD_SIZE) >>
  (data)
  )
}

fn read_string_known_length<'a>(input: &'a [u8], length: i32) -> IResult<&[u8], Cow<'a, str>> {
    map!(input, take!(length), String::from_utf8_lossy)
}

pub fn read_nastran_string<'a>(input: &'a [u8]) -> IResult<&[u8], Cow<'a, str>> {
    do_parse!(input,
  length: read_fortran_i32 >>
  apply!(read_known_i32,length*WORD_SIZE) >>
  data: take!(length*WORD_SIZE) >>
  apply!(read_known_i32,length*WORD_SIZE) >>
  (String::from_utf8_lossy(data))
  )
}

pub fn read_nastran_string_known_length<'a>(input: &'a [u8],
                                        length: i32)
                                        -> IResult<&[u8], Cow<'a, str>> {
    do_parse!(input,
  apply!(read_fortran_known_i32,length) >>
  apply!(read_known_i32,length*WORD_SIZE) >>
  data: take!(length*WORD_SIZE) >>
  apply!(read_known_i32,length*WORD_SIZE) >>
  (String::from_utf8_lossy(data))
  )
}

named!(pub read_nastran_key<i32>, do_parse!(
  apply!(read_known_i32,4) >>
  data: le_i32 >>
  apply!(read_known_i32,4) >>
  (data)
));

pub fn read_nastran_known_key(input: &[u8], v: i32) -> IResult<&[u8], ()> {
    do_parse!(input,
  apply!(read_known_i32,4) >>
  apply!(read_known_i32,v) >>
  apply!(read_known_i32,4) >>
  ()
)
}

fn buf_to_struct<T: Sized>(buf: &[u8]) -> &T {
    unsafe { transmute(buf.as_ptr()) }
}

named!(read_header<FileHeader>,
  do_parse!(
  date: apply!(read_nastran_data_known_length, 3) >>
  apply!(read_nastran_tag,b"NASTRAN FORT TAPE ID CODE - ") >>
  label: apply!(read_nastran_data_known_length,2) >>
  apply!(read_nastran_known_key,-1) >>
  apply!(read_nastran_known_key,0) >>
  (FileHeader {date:buf_to_struct(date), label: label})
  )
  );

named!(pub read_first_table_record, do_parse!(
  record: read_nastran_data >>
  apply!(read_nastran_known_key,-3) >>
  (record)
));

fn read_negative_i32(input: &[u8]) -> IResult<&[u8], &i32> {
    map!(input,
  recognize!(
    bits!(
    do_parse!(
    tag_bits!(u8,1,0b1) >>
    take_bits!(u32,31) >>
    ())
  )),|v| buf_to_struct(v) )
}

named!(pub read_nastran_eof<()>, apply!(read_fortran_known_i32,0));

named!(pub read_nastran_eor<&i32>,do_parse!(
  apply!(read_known_i32,4) >>
  value: read_negative_i32 >>
  apply!(read_known_i32,4) >>
  (value)
));

named!(pub read_last_table_record<()>,do_parse!(
  apply!(read_nastran_known_i32,0) >>
  read_nastran_eof >>
  ()
));

named!(read_table_record,do_parse!(
  apply!(read_nastran_known_i32,0) >>
  data : read_nastran_data >>
  read_nastran_eor >>
  (data)
));

pub struct DataBlockStart<'a> {
    pub name: Cow<'a, str>,
    pub trailer: DataBlockTrailer<'a>,
    pub record_type: DataBlockType,
}

#[derive(Debug)]
pub struct GenericDataBlock<'a> {
    pub name: Cow<'a, str>,
    pub trailer: DataBlockTrailer<'a>,
    pub record_type: DataBlockType,
    pub header: &'a [u8],
    pub records: Vec<&'a [u8]>,
}

#[derive(Debug)]
pub struct DataBlockIdentPair<'a, T: 'a, U: 'a> {
    pub name: Cow<'a, str>,
    pub trailer: DataBlockTrailer<'a>,
    pub record_type: DataBlockType,
    pub header: &'a [u8],
    pub record_pairs: Vec<(&'a T, &'a [U])>,
}

pub struct OUGIdent {
    pub acode: i32,
    pub tcode: i32,
    pub datcod: i32,
    pub subcase: i32,
    pub var1: [u8; 12],
    pub rcode: i32,
    pub fcode: i32,
    pub numwde: i32,
    pub undef1: [i32; 2],
    pub acflag: i32,
    pub undef2: [i32; 3],
    pub rmssf: f32,
    pub undef3: [i32; 5],
    pub thermal: i32,
    pub undef4: [i32; 27],
    pub title: [u8; 128],
    pub subtitl: [u8; 128],
    pub label: [u8; 128],
}

pub struct OUGData {
    pub ekey: i32,
    pub etype: i32,
    pub data: [f32; 12],
}

type OUG<'a> = DataBlockIdentPair<'a, OUGIdent, OUGData>;

pub struct OEFIdent {
    pub acode: i32,
    pub tcode: i32,
    pub eltype: i32,
    pub subcase: i32,
    pub var1: [u8; 12],
    pub dloadid: i32,
    pub fcode: i32,
    pub numwde: i32,
    pub ocode: i32,
    pub pid: i32,
    pub undef1: i32,
    pub q4cstr: i32,
    pub plsloc: i32,
    pub undef2: i32,
    pub rmssf: f32,
    pub undef3: [i32; 5],
    pub thermal: i32,
    pub undef4: [i32; 27],
    pub title: [u8; 128],
    pub subtitl: [u8; 128],
    pub label: [u8; 128],
}

enum OEFValues {

}

pub struct CRODForce {
    var: i32,
    af: f32,
    trq: f32,
}

pub enum OEFData {
    CROD(CRODForce),
}

type OEF<'a> = DataBlockIdentPair<'a, OEFIdent, OEFData>;

named!(pub read_datablock_start<DataBlockStart>,do_parse!(
  name: apply!(read_nastran_string_known_length,2) >>
  apply!(read_nastran_known_key,-1) >>
  trailer: apply!(read_nastran_data_known_length,7) >>
  apply!(read_nastran_known_key,-2) >>
  record_type: alt!(
    apply!(read_nastran_known_i32,0) => {|_| DataBlockType::Table}
    | apply!(read_nastran_known_i32,1) => {|_| DataBlockType::Matrix}
    | apply!(read_nastran_known_i32,2) => {|_| DataBlockType::StringFactor}
    | apply!(read_nastran_known_i32,3) => {|_| DataBlockType::MatrixFactor}
  ) >>
(DataBlockStart {name:name,trailer:buf_to_struct(trailer),record_type:record_type})
));

named!(pub read_datablock_header,do_parse!(
  header: read_nastran_data >>
  apply!(read_nastran_known_key,-3) >>
  (header)
));

fn read_generic_datablock<'a>(input: &'a [u8],
                              start: DataBlockStart<'a>)
                              -> IResult<&'a [u8], DataBlock<'a>> {
    let (input, header) = try_parse!(input,read_datablock_header);
    let (input, records) = try_parse!(input,many0!(read_table_record));
    let (input, _) = try_parse!(input,read_last_table_record);
    IResult::Done(input,
                  DataBlock::Generic(GenericDataBlock {
                                         name: start.name,
                                         trailer: start.trailer,
                                         record_type: start.record_type,
                                         header: header,
                                         records: records,
                                     }))
}
impl fmt::Display for OUGIdent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let title = String::from_utf8_lossy(&self.title);
        let subtitle = String::from_utf8_lossy(&self.subtitl);
        let label = String::from_utf8_lossy(&self.label);
        write!(f, "OUG_IDENT[");
        write!(f, "acode={},", self.acode);
        write!(f, "tcode={},", self.tcode);
        write!(f, "title=\"{}\",", title);
        write!(f, "subtitle=\"{}\",", subtitle);
        write!(f, "label=\"{}\",", label);
        write!(f, "]")
    }
}


fn read_ident<T>(input: &[u8]) -> IResult<&[u8], &T> {
    let struct_size: i32 = (size_of::<T>() / 4) as i32;
    let (input, _) = try_parse!(input,apply!(read_nastran_known_i32,0));
    let (input, data) = try_parse!(input,apply!(read_nastran_data_known_length,struct_size));
    let (input, _) = try_parse!(input,read_nastran_eor);
    IResult::Done(input, buf_to_struct(data))
}

fn read_data<T>(input: &[u8]) -> IResult<&[u8], &[T]> {
    let struct_size: i32 = (size_of::<T>() / 4) as i32;
    let (input, _) = try_parse!(input,apply!(read_nastran_known_i32,0));
    let (input, data) = try_parse!(input,read_nastran_data);
    let (input, _) = try_parse!(input,read_nastran_eor);
    if data.len() % size_of::<T>() != 0 {
        return IResult::Error(ErrorKind::Custom(1));
    }
    let count = data.len() / size_of::<T>();
    let sl = unsafe { from_raw_parts::<T>(transmute(data.as_ptr()), count) };
    IResult::Done(input, sl)
}

fn read_OUG_datablock<'a>(input: &'a [u8],
                          start: DataBlockStart<'a>)
                          -> IResult<&'a [u8], DataBlock<'a>> {
    let (input, header) = try_parse!(input,read_datablock_header);
    let (input, record_pairs) =
        try_parse!(input,many0!(pair!(read_ident::<OUGIdent>,read_data::<OUGData>)));
    let (input, _) = try_parse!(input,read_last_table_record);
    IResult::Done(input,
                  DataBlock::OUG(OUG {
                                     name: start.name,
                                     trailer: start.trailer,
                                     record_type: start.record_type,
                                     header: header,
                                     record_pairs: record_pairs,
                                 }))
}


pub fn read_struct_array<'a, T>(input: &'a [u8], count: usize) -> IResult<&'a [u8], &'a [T]> {
    let length = size_of::<T>() * count;
    let (input, data) = try_parse!(input,take!(length));
    let sl = unsafe { from_raw_parts::<T>(transmute(data.as_ptr()), count) };
    return IResult::Done(input, sl);
}

fn read_datablock(input: &[u8]) -> IResult<&[u8], DataBlock> {
    let (input, start) = try_parse!(input,read_datablock_start);
    let table_name = start.name.clone().into_owned();
    match table_name.as_str() {
        "OUGV1   " => read_OUG_datablock(input, start),
        "GEOM1S  " => geom1::read_datablock(input, start),
        _ => read_generic_datablock(input, start),
    }
}

named!(pub read_op2<OP2>,do_parse!(
  header: read_header >>
  blocks: many0!(read_datablock) >>
  read_nastran_eof >>
  eof!() >>
  (OP2 {header:header,blocks:blocks})
));
