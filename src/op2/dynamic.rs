
use op2;
use op2::keyed;
use nom::IResult;
use ascii::AsciiChar;

pub type DataBlock<'a> = keyed::DataBlock<'a, Record<'a>>;

#[derive(Debug)]
pub struct EIGR {
    sid: i32,
    method: [AsciiChar; 8],
    f1: f32,
    f2: f32,
    ne: i32,
    nd: i32,
    undef1: [i32; 2],
    norm: [AsciiChar; 8],
    g: i32,
    c: i32,
    undef2: [i32; 5],
}

pub type EIGCs<'a> = &'a [u8];

#[derive(Debug)]
pub enum Record<'a> {
    EIGR(&'a [EIGR]),
    EIGC(EIGCs<'a>),
    Unknown(keyed::Key, keyed::UnknownRecord<'a>),
}

named!(read_record<Record>,
    switch!(call!(keyed::read_record_key),
      keyed::RecordKey { key: (307,3,85), size } => map!(apply!(keyed::read_fixed_size_record,size),Record::EIGR) |
      keyed::RecordKey { key: (207,2,87), size } => map!(apply!(keyed::read_variable_record,size),Record::EIGC) |
      keyed::RecordKey { key, size } => map!(apply!(keyed::read_unknown_record,size),|r| Record::Unknown(key,r) )
    ) 
);

pub fn read_datablock<'a>(input: &'a [u8],
                          start: op2::DataBlockStart<'a>)
                          -> IResult<&'a [u8], DataBlock<'a>> {
    let (input, header) = try_parse!(input,op2::read_datablock_header);
    let (input, (records, _)) = try_parse!(input,many_till!(read_record,keyed::read_eodb));
    let (input, _) = try_parse!(input,op2::read_last_table_record);
    IResult::Done(input,
                  DataBlock {
                      name: start.name,
                      trailer: start.trailer,
                      record_type: start.record_type,
                      header: header,
                      records: records,
                  })
}