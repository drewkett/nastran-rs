
use nom::{IResult, le_i32};
use std::mem::size_of;

use op2;

#[derive(Debug)]
pub struct DataBlock<'a, T: 'a> {
    pub name: &'a str,
    pub trailer: op2::DataBlockTrailer<'a>,
    pub record_type: op2::DataBlockType,
    pub header: &'a [u8],
    pub records: Vec<T>,
}

#[derive(Debug)]
pub struct UnknownRecord<'a> {
    key: [i32; 3],
    data: &'a [u8],
}

pub fn read_unknown_record(input: &[u8]) -> IResult<&[u8], UnknownRecord> {
    let (input, _) = try_parse!(input, apply!(op2::read_nastran_known_i32, 0));
    let (input, record_size) = try_parse!(input, op2::read_fortran_i32);
    let (input, _) = try_parse!(input, apply!(op2::read_known_i32, record_size * 4));
    let (input, v1) = try_parse!(input, le_i32);
    let (input, v2) = try_parse!(input, le_i32);
    let (input, v3) = try_parse!(input, le_i32);
    let remaining = record_size - 3;
    let (input, data) = try_parse!(input, take!(remaining * 4));
    let (input, _) = try_parse!(input, apply!(op2::read_known_i32, record_size * 4));
    let (input, _) = try_parse!(input, op2::read_nastran_eor);
    return IResult::Done(input,
                         UnknownRecord {
                             key: [v1, v2, v3],
                             data: data,
                         });
}

pub fn read_record<T>(input: &[u8], v1: i32, v2: i32, v3: i32) -> IResult<&[u8], &[T]> {
    let (input, _) = try_parse!(input, apply!(op2::read_nastran_known_i32, 0));
    let (input, record_size) = try_parse!(input, op2::read_fortran_i32);
    let (input, _) = try_parse!(input, apply!(op2::read_known_i32, record_size * 4));
    let (input, _) = try_parse!(input, apply!(op2::read_known_i32, v1));
    let (input, _) = try_parse!(input, apply!(op2::read_known_i32, v2));
    let (input, _) = try_parse!(input, apply!(op2::read_known_i32, v3));
    let struct_size = (size_of::<T>() / 4) as i32;
    let count = if struct_size > 0 {
        (record_size - 3) / struct_size
    } else {
        0
    };
    let (input, data) = try_parse!(input, apply!(op2::read_struct_array::<T>, count as usize));
    let (input, _) = try_parse!(input, apply!(op2::read_known_i32, record_size * 4));
    let (input, _) = try_parse!(input, op2::read_nastran_eor);
    return IResult::Done(input, data);
}

named!(pub read_eodb<()>,value!((),apply!(read_record::<()>,65535,65535,65535)));