
use std::slice::from_raw_parts;
use std::mem::{size_of, transmute};

use nom::{IResult, ErrorKind};

use op2;

#[derive(Debug)]
pub struct DataBlockIdentPair<'a, T: 'a, U: 'a> {
    pub name: &'a str,
    pub trailer: op2::DataBlockTrailer<'a>,
    pub record_type: op2::DataBlockType,
    pub header: &'a [u8],
    pub record_pairs: Vec<(&'a T, &'a [U])>,
}

pub fn read_ident<T>(input: &[u8]) -> IResult<&[u8], &T> {
    let struct_size: i32 = (size_of::<T>() / 4) as i32;
    let (input, _) = try_parse!(input, apply!(op2::read_nastran_known_i32, 0));
    let (input, data) = try_parse!(
        input,
        apply!(op2::read_nastran_data_known_length, struct_size)
    );
    let (input, _) = try_parse!(input, op2::read_nastran_eor);
    IResult::Done(input, op2::buf_to_struct(data))
}

pub fn read_data<T>(input: &[u8]) -> IResult<&[u8], &[T]> {
    let (input, _) = try_parse!(input, apply!(op2::read_nastran_known_i32, 0));
    let (input, data) = try_parse!(input, op2::read_nastran_data);
    let (input, _) = try_parse!(input, op2::read_nastran_eor);
    if data.len() % size_of::<T>() != 0 {
        return IResult::Error(ErrorKind::Custom(21));
    }
    let count = data.len() / size_of::<T>();
    let sl = unsafe { from_raw_parts::<T>(transmute(data.as_ptr()), count) };
    IResult::Done(input, sl)
}
