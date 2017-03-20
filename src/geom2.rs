
use op2;
use keyed;
use keyed::{read_unknown_record};
use nom::{IResult};

#[derive(Debug)]
pub struct CBUSH {
    eid: i32,
    pid: i32,
    ga: i32,
    gb: i32,
    var1: [i32;4],
    cid: i32,
    s: f32,
    ocid: i32,
    s1: f32,
    s2: f32,
    s3: f32,
}

pub type DataBlock<'a> = keyed::DataBlock<'a, Record<'a>>;

#[derive(Debug)]
enum Record<'a> {
    CBUSH(&'a [CBUSH]),
    Unknown(keyed::UnknownRecord<'a>),
}

named!(read_record<Record>,
    alt!(
      apply!(keyed::read_record::<CBUSH>,2608,26,60) => { |s| Record::CBUSH(s) }
      | read_unknown_record => { |r| Record::Unknown(r) }
    )
    );

pub fn read_datablock<'a>(input: &'a [u8],
                            start: op2::DataBlockStart<'a>)
                            -> IResult<&'a [u8], op2::DataBlock<'a>> {
    let (input, header) = try_parse!(input,op2::read_datablock_header);
    let (input, (records, _)) = try_parse!(input,many_till!(read_record,keyed::read_eodb));
    let (input, _) = try_parse!(input,op2::read_last_table_record);
    IResult::Done(input,
                  op2::DataBlock::GEOM2(DataBlock {
                                       name: start.name,
                                       trailer: start.trailer,
                                       record_type: start.record_type,
                                       header: header,
                                       records: records,
                                   }))
}