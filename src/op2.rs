use std::mem::{size_of, transmute};
use std::slice::from_raw_parts;
use std::marker::Sized;

use nom::{IResult, Needed, be_i64, le_i64, le_i32, le_i8};

named!(read_fortran_chunk,
       do_parse!(
  length: le_i32 >>
  data: take!(length) >>
  tag!(unsafe { transmute::<i32,[u8;4]>(length)}) >>
  (data)
));

fn i32_to_bytearray(num: i32) -> [u8; 4] {
    unsafe { transmute(num.to_le()) }
}

fn read_known_i32(input: &[u8], v: i32) -> IResult<&[u8], ()> {
    tag!(input, i32_to_bytearray(v)).map(|v| ())
}

named!(read_fortran_i32<i32>,
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

named!(read_nastran_i32<i32>,
  do_parse!(
  apply!(read_fortran_known_i32,1) >>
  value: read_fortran_i32 >>
  (value)
  )
);

fn read_nastran_known_i32(input: &[u8], v: i32) -> IResult<&[u8], ()> {
    do_parse!(input,
  apply!(read_fortran_known_i32,1) >>
  apply!(read_fortran_known_i32,v) >>
  ()
  )
}

const WORD_SIZE: i32 = 4;

fn read_nastran_tag<'a>(input: &'a [u8], v: &[u8]) -> IResult<&'a [u8], ()> {
    let l: i32 = v.len() as i32;
    do_parse!(input,
  apply!(read_fortran_known_i32,l/WORD_SIZE) >>
  apply!(read_known_i32,l) >>
  tag!(v) >>
  apply!(read_known_i32,l) >>
  ()
  )
}

fn read_nastran_data_known_length(input: &[u8], v: i32) -> IResult<&[u8], &[u8]> {
    do_parse!(input,
  apply!(read_fortran_known_i32,v) >>
  apply!(read_known_i32,v*WORD_SIZE) >>
  data: take!(v*WORD_SIZE) >>
  apply!(read_known_i32,v*WORD_SIZE) >>
  (data)
  )
}

fn read_nastran_data(input: &[u8]) -> IResult<&[u8], &[u8]> {
    do_parse!(input,
  length: read_fortran_i32 >>
  apply!(read_known_i32,length*WORD_SIZE) >>
  data: take!(length*WORD_SIZE) >>
  apply!(read_known_i32,length*WORD_SIZE) >>
  (data)
  )
}

named!(read_nastran_key<i32>, do_parse!(
  apply!(read_known_i32,4) >>
  data: le_i32 >>
  apply!(read_known_i32,4) >>
  (data)
));

fn read_nastran_known_key(input: &[u8], v: i32) -> IResult<&[u8], ()> {
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

#[derive(Debug)]
struct HeaderDate {
    month: i32,
    day: i32,
    year: i32,
}

#[derive(Debug)]
pub struct Header<'a> {
    date: &'a HeaderDate,
    label: &'a [u8], // Might want to make this fixed length at some point
}

named!(read_header<Header>,
  do_parse!(
  date: apply!(read_nastran_data_known_length, 3) >>
  apply!(read_nastran_tag,b"NASTRAN FORT TAPE ID CODE - ") >>
  label: apply!(read_nastran_data_known_length,2) >>
  apply!(read_nastran_known_key,-1) >>
  apply!(read_nastran_known_key,0) >>
  (Header {date:buf_to_struct(date), label: label})
  )
  );

#[derive(Debug)]
enum DataBlockType {
    Table,
    Matrix,
    StringFactor,
    MatrixFactor,
}

#[derive(Debug)]
pub struct DataBlockHeader<'a> {
    name: &'a [u8],
    trailer: &'a [u8],
    record_type: DataBlockType,
    name2: &'a [u8],
}

named!(read_trailer<DataBlockHeader>,do_parse!(
  name: apply!(read_nastran_data_known_length,2) >>
  apply!(read_nastran_known_key,-1) >>
  trailer: apply!(read_nastran_data_known_length,7) >>
  apply!(read_nastran_known_key,-2) >>
  record_type: alt!(
    apply!(read_nastran_known_i32,0) => {|_| DataBlockType::Table}
    | apply!(read_nastran_known_i32,1) => {|_| DataBlockType::Matrix}
    | apply!(read_nastran_known_i32,2) => {|_| DataBlockType::StringFactor}
    | apply!(read_nastran_known_i32,3) => {|_| DataBlockType::MatrixFactor}
  ) >>
  //Book claims this always should be length 2, doesn't appear to be the case
  name2: read_nastran_data >>
  apply!(read_nastran_known_key,-3) >>
(DataBlockHeader {name:name,trailer:trailer,record_type:record_type,name2:name2})
));

fn read_negative_i32(input: &[u8]) -> IResult<&[u8], ()> {
    bits!(input,
    do_parse!(
    tag_bits!(u8,1,0b1) >>
    take_bits!(u32,31) >>
    ())
  )
}

named!(read_nastran_eof<()>, apply!(read_fortran_known_i32,0));

named!(read_nastran_eor<()>,do_parse!(
  apply!(read_known_i32,4) >>
  read_negative_i32 >>
  apply!(read_known_i32,4) >>
  ()
));

named!(read_last_table_record<()>,do_parse!(
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

named!(read_table_records<Vec<&[u8]>>,
map!(
  many_till!(read_table_record,read_last_table_record),
  |(records,_)| records // Extract Records since last table record is null
));

named!(read_table<DataBlock>, do_parse!(
  trailer: read_trailer >>
  records: read_table_records >>
  (DataBlock { header: trailer, records: records })
));

named!(read_tables<Vec<DataBlock>>,
map!(
  many_till!(read_table,read_nastran_eof),
  |(tables,_)| tables
  ));

#[derive(Debug)]
pub struct DataBlock<'a> {
    pub header: DataBlockHeader<'a>,
    pub records: Vec<&'a [u8]>,
}

#[derive(Debug)]
pub struct OP2<'a> {
    pub header: Header<'a>,
    pub blocks: Vec<DataBlock<'a>>,
}

named!(pub read_op2<OP2>,do_parse!(
  header: read_header >>
  blocks: read_tables >>
  eof!() >>
  (OP2 {header:header,blocks:blocks})
));

// pub fn read_op2(mut buf: &[u8]) {
//   let (new_buf,c) = read_header(buf).unwrap();
//   buf = new_buf;
//   println!("{:?}",c);
//   let (buf,tables) = read_tables(buf).unwrap();
//   for table  in tables {
//     println!("{:?}",table.header);
//   }
//   // let (buf,c) = read_nastran_data(buf).unwrap();
//   // println!("{:?}",String::from_utf8_lossy(c));
// }
