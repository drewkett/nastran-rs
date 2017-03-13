use std::mem::{size_of,transmute};
use nom::{IResult, Needed, be_i64, le_i64,le_i32};
use std::slice::from_raw_parts;
use std::marker::Sized;

named!(read_chunk, do_parse!(
  length: le_i32 >>
  data: take!(length) >>
  tag!(unsafe { transmute::<i32,[u8;4]>(length)}) >>
  (data)
));

#[derive(Debug)]
struct ABC {
  a: i32,
  b: i32
}

named!(read_fixed, do_parse!(
  length: le_i32 >>
  data: take!(length) >>
  tag!(unsafe { transmute::<i32,[u8;4]>(length)}) >>
  (data)
));

fn buf_to_struct<T: Sized>(buf: &[u8]) -> &T{
  unsafe {
    transmute(buf.as_ptr())
  }
}

pub fn read_op2(buf: &[u8]) {
  let (buf,b) = read_chunk(buf).unwrap();
  println!("{:?}",b);
  let c:&ABC = buf_to_struct(b);
  println!("{:?}",c);
}