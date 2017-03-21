
use op2;
use keyed;
use nom::IResult;

pub type DataBlock<'a> = keyed::DataBlock<'a, Record<'a>>;

#[derive(Debug)]
pub struct PBUSH {
    pid: i32,
    k: [f32; 6],
    b: [f32; 6],
    ge: f32,
    sa: f32,
    st: f32,
    ea: f32,
    et: f32,
}

#[derive(Debug)]
pub enum Record<'a> {
    PBUSH(&'a [PBUSH]),
    Unknown(keyed::Key, keyed::UnknownRecord<'a>),
}

named!(read_record<Record>,
    switch!(call!(keyed::read_record_key),
      keyed::RecordKey { key: (1402,14,37), size } => map!(apply!(keyed::read_fixed_size_record,size),Record::PBUSH) |
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