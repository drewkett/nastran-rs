use std::mem::{size_of,transmute};
use nom::{IResult, Needed, be_i64, le_i64,le_i32};
use std::slice::from_raw_parts;
use std::marker::Sized;

named!(read_fortran_chunk, do_parse!(
  length: le_i32 >>
  data: take!(length) >>
  tag!(unsafe { transmute::<i32,[u8;4]>(length)}) >>
  (data)
));

fn i16_to_bytearray(num: i16) -> [u8;2] {
  unsafe { transmute(num.to_le()) }
}

fn i32_to_bytearray(num: i32) -> [u8;4] {
  unsafe { transmute(num.to_le()) }
}

fn read_known_i16(input: &[u8], v: i16) -> IResult<&[u8],()> {
  tag!(input,i16_to_bytearray(v)).map(|v| ())
}

fn read_known_i32(input: &[u8], v: i32) -> IResult<&[u8],()> {
  tag!(input,i32_to_bytearray(v)).map(|v| ())
}

fn read_fortran_i32(input: &[u8]) -> IResult<&[u8],i32> {
  do_parse!(input,
  apply!(read_known_i32,4) >>
  v: le_i32 >>
  apply!(read_known_i32,4) >>
  (v)
  )
}

fn read_fortran_known_i32(input: &[u8], v: i32) -> IResult<&[u8],()> {
  do_parse!(input,
  apply!(read_known_i32,4) >>
  apply!(read_known_i32,v) >>
  apply!(read_known_i32,4) >>
  ()
  )
}

fn read_nastran_i32(input: &[u8]) -> IResult<&[u8],i32> {
  do_parse!(input,
  apply!(read_fortran_known_i32,1) >>
  value: read_fortran_i32 >>
  (value)
  )
}

fn read_nastran_known_i32(input: &[u8], v: i32) -> IResult<&[u8],()> {
  do_parse!(input,
  apply!(read_fortran_known_i32,1) >>
  apply!(read_fortran_known_i32,v) >>
  ()
  )
}

const WORD_SIZE : i32 = 4;

fn read_nastran_tag<'a>(input: &'a[u8], v: &[u8]) -> IResult<&'a[u8],()> {
  let l: i32 = v.len() as i32;
  do_parse!(input,
  apply!(read_fortran_known_i32,l/WORD_SIZE) >>
  apply!(read_known_i32,l) >>
  tag!(v) >>
  apply!(read_known_i32,l) >>
  ()
  )
}

fn read_nastran_known_length(input: &[u8], v: i32) -> IResult<&[u8], &[u8]> {
  do_parse!(input,
  apply!(read_fortran_known_i32,v) >>
  apply!(read_known_i32,v*WORD_SIZE) >>
  data: take!(v*WORD_SIZE) >>
  apply!(read_known_i32,v*WORD_SIZE) >>
  (data)
  )
}

fn read_nastran_string(input: &[u8]) -> IResult<&[u8], &[u8]> {
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

fn read_nastran_known_key(input: &[u8], v: i32) -> IResult<&[u8],()> {
 do_parse!(input,
  apply!(read_known_i32,4) >>
  apply!(read_known_i32,v) >>
  apply!(read_known_i32,4) >>
  ()
)
}

fn buf_to_struct<T: Sized>(buf: &[u8]) -> &T{
  unsafe {
    transmute(buf.as_ptr())
  }
}

#[derive(Debug)]
struct HeaderDate {
  month: i32,
  day: i32,
  year: i32,
}

#[derive(Debug)]
struct Header <'a> {
  date: &'a HeaderDate,
  label: &'a [u8], // Might want to make this fixed length at some point
}

named!(read_header<Header>,
  do_parse!(
  date: apply!(read_nastran_known_length, 3) >>
  apply!(read_nastran_tag,b"NASTRAN FORT TAPE ID CODE - ") >>
  label: apply!(read_nastran_known_length,2) >>
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
struct DataBlock <'a> {
name: &'a [u8],
trailer: &'a [u8],
record_type: DataBlockType,
name2: &'a [u8],
}

named!(read_trailer<DataBlock>,do_parse!(
  name: apply!(read_nastran_known_length,2) >>
  apply!(read_nastran_known_key,-1) >>
  trailer: apply!(read_nastran_known_length,7) >>
  apply!(read_nastran_known_key,-2) >>
  record_type: alt!(
    apply!(read_nastran_known_i32,0) => {|_| DataBlockType::Table}
    | apply!(read_nastran_known_i32,1) => {|_| DataBlockType::Matrix}
    | apply!(read_nastran_known_i32,2) => {|_| DataBlockType::StringFactor}
    | apply!(read_nastran_known_i32,3) => {|_| DataBlockType::MatrixFactor}
  ) >>
  name2: apply!(read_nastran_known_length,2) >>
  apply!(read_nastran_known_key,-3) >>
(DataBlock {name:name,trailer:trailer,record_type:record_type,name2:name2})
));

pub fn read_op2(buf: &[u8]) {
  let (buf,c) = read_header(buf).unwrap();
  println!("{:?}",c);
  let (buf,c) = read_trailer(buf).unwrap();
  println!("{:?}",c);
  let (buf,c) = read_nastran_string(buf).unwrap();
  println!("{:?}",c);
  let (buf,c) = read_nastran_string(buf).unwrap();
  println!("{:?}",c);
  // println!("{:?}",String::from_utf8_lossy(c));
  // let (buf,c) = read_nastran_known_key(buf,-1).unwrap();
}