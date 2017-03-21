
use op2;
use keyed;
use nom::IResult;

pub type DataBlock<'a> = keyed::DataBlock<'a, Record<'a>>;

#[derive(Debug)]
pub struct EIGR {
    sid: i32,
    method: [u8; 8],
    f1: f32,
    f2: f32,
    ne: i32,
    nd: i32,
    undef1: [i32; 2],
    norm: [u8; 8],
    g: i32,
    c: i32,
    undef2: [i32; 5],
}

#[derive(Debug)]
enum Record<'a> {
    Unknown(keyed::UnknownRecord<'a>),
    EIGR(&'a [EIGR]),
}

named!(read_record<Record>,
    alt!(
      apply!(keyed::read_record::<EIGR>,307,3,85) => { |s| Record::EIGR(s) }
      | call!(keyed::read_unknown_record) => { |r| Record::Unknown(r) }
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