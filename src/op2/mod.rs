use std::mem::{size_of, transmute};
use std::slice::from_raw_parts;
use std::marker::Sized;

use nom::{IResult, le_i32};

mod geom1;
mod geom2;
mod geom4;
mod ept;
mod dynamic;
mod keyed;
mod ident;
mod oug;
mod generic;

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
    Generic(generic::DataBlock<'a>),
    OUG(oug::DataBlock<'a>),
    GEOM1(geom1::DataBlock<'a>),
    GEOM2(geom2::DataBlock<'a>),
    GEOM4(geom4::DataBlock<'a>),
    EPT(ept::DataBlock<'a>),
    DYNAMIC(dynamic::DataBlock<'a>),
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
    tag!(input, i32_to_bytearray(v)).map(|_| ())
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
              apply!(read_known_i32, 4) >> apply!(read_known_i32, v) >>
              apply!(read_known_i32, 4) >> ())
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
              apply!(read_fortran_known_i32, 1) >> apply!(read_fortran_known_i32, v) >> ())
}

const WORD_SIZE: i32 = 4;

pub fn read_nastran_tag<'a>(input: &'a [u8], v: &[u8]) -> IResult<&'a [u8], ()> {
    let l: i32 = v.len() as i32;
    do_parse!(input,
              apply!(read_fortran_known_i32, l / WORD_SIZE) >>
              apply!(read_known_i32, l) >> tag!(v) >> apply!(read_known_i32, l) >> ())
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

pub fn read_nastran_string<'a>(input: &'a [u8]) -> IResult<&[u8], &str> {
    do_parse!(input,
  length: read_fortran_i32 >>
  apply!(read_known_i32,length*WORD_SIZE) >>
  data: take_str!(length*WORD_SIZE) >>
  apply!(read_known_i32,length*WORD_SIZE) >>
  (data)
  )
}

pub fn read_nastran_string_known_length<'a>(input: &'a [u8], length: i32) -> IResult<&[u8], &str> {
    do_parse!(input,
  apply!(read_fortran_known_i32,length) >>
  apply!(read_known_i32,length*WORD_SIZE) >>
  data: take_str!(length*WORD_SIZE) >>
  apply!(read_known_i32,length*WORD_SIZE) >>
  (data)
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
              apply!(read_known_i32, 4) >> apply!(read_known_i32, v) >>
              apply!(read_known_i32, 4) >> ())
}

pub fn buf_to_struct<T: Sized>(buf: &[u8]) -> &T {
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

pub struct DataBlockStart<'a> {
    pub name: &'a str,
    pub trailer: DataBlockTrailer<'a>,
    pub record_type: DataBlockType,
}

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


pub fn read_struct_array<'a, T>(input: &'a [u8], count: usize) -> IResult<&'a [u8], &'a [T]> {
    let length = size_of::<T>() * count;
    let (input, data) = try_parse!(input,take!(length));
    let sl = unsafe { from_raw_parts::<T>(transmute(data.as_ptr()), count) };
    return IResult::Done(input, sl);
}

fn read_datablock(input: &[u8]) -> IResult<&[u8], DataBlock> {
    let (input, start) = try_parse!(input,read_datablock_start);
    match start.name {
        "OUGV1   " => map!(input,apply!(oug::read_datablock,start),DataBlock::OUG),
        "GEOM1S  " => map!(input,apply!(geom1::read_datablock,start),DataBlock::GEOM1),
        "GEOM2S  " => map!(input,apply!(geom2::read_datablock,start),DataBlock::GEOM2),
        "GEOM4S  " => map!(input,apply!(geom4::read_datablock,start),DataBlock::GEOM4),
        "EPTS    " => map!(input,apply!(ept::read_datablock,start),DataBlock::EPT),
        "DYNAMICS" => map!(input,apply!(dynamic::read_datablock,start),DataBlock::DYNAMIC),
        _ => map!(input,apply!(generic::read_datablock,start),DataBlock::Generic),
    }
}

named!(pub read_op2<OP2>,do_parse!(
  header: read_header >>
  blocks: many0!(read_datablock) >>
  read_nastran_eof >>
  eof!() >>
  (OP2 {header:header,blocks:blocks})
));
