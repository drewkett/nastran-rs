
use op2;
use op2::keyed;
use nom::IResult;

pub type DataBlock<'a> = keyed::DataBlock<'a, Record<'a>>;

type RBE2s<'a> = &'a [u8];

#[derive(Debug)]
pub enum Record<'a> {
    RBE2(RBE2s<'a>),
    Unknown(keyed::Key, keyed::UnknownRecord<'a>),
}

named!(read_record<Record>,
    switch!(call!(keyed::read_record_key),
      keyed::RecordKey { key: (6901,69,295), size } => map!(
          apply!(keyed::read_variable_record,size),
          Record::RBE2) |
      keyed::RecordKey { key, size } => map!(
          apply!(keyed::read_unknown_record,size),
          |r| Record::Unknown(key,r) )
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
