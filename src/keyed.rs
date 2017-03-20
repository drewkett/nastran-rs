
use nom::{IResult};
use std::mem::{size_of, transmute};
use std::borrow::Cow;

use op2;

#[derive(Debug)]
pub struct DataBlock<'a, T: 'a> {
    pub name: Cow<'a, str>,
    pub trailer: op2::DataBlockTrailer<'a>,
    pub record_type: op2::DataBlockType,
    pub header: &'a [u8],
    pub records: Vec<T>,
}

pub fn read_record<T>(input: &[u8], v1: i32, v2: i32, v3: i32) -> IResult<&[u8], &[T]> {
    let (input, _) = try_parse!(input,apply!(op2::read_nastran_known_i32,0));
    let (input, record_size) = try_parse!(input,op2::read_fortran_i32);
    let (input, _) = try_parse!(input,apply!(op2::read_known_i32,record_size*4));
    let (input, _) = try_parse!(input,apply!(op2::read_known_i32,v1));
    let (input, _) = try_parse!(input,apply!(op2::read_known_i32,v2));
    let (input, _) = try_parse!(input,apply!(op2::read_known_i32,v3));
    let struct_size = (size_of::<T>() / 4) as i32;
    let count = if struct_size > 0 {
        (record_size - 3) / struct_size
    } else {
        0
    };
    let (input, data) = try_parse!(input,apply!(op2::read_struct_array::<T>,count as usize));
    let (input, _) = try_parse!(input,apply!(op2::read_known_i32,record_size*4));
    let (input, _) = try_parse!(input,op2::read_nastran_eor);
    return IResult::Done(input, data);
}