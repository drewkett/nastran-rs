
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

#[derive(Debug)]
pub struct CDAMP2 {
    eid: i32,
    b: f32,
    g1: i32,
    g2: i32,
    c1: i32,
    c2: i32,
}

#[derive(Debug)]
pub struct CONM2 {
    eid: i32,
    g: i32,
    cid: i32,
    m: f32,
    x1: f32,
    x2: f32,
    x3: f32,
    I11: f32,
    I21: f32,
    I22: f32,
    I31: f32,
    I32: f32,
    I33: f32,
}

pub type DataBlock<'a> = keyed::DataBlock<'a, Record<'a>>;

#[derive(Debug)]
enum Record<'a> {
    CBUSH(&'a [CBUSH]),
    CDAMP2(&'a [CDAMP2]),
    CONM2(&'a [CONM2]),
    Unknown(keyed::UnknownRecord<'a>),
}

named!(read_record<Record>,
    alt!(
      apply!(keyed::read_record::<CBUSH>,2608,26,60) => { |s| Record::CBUSH(s) }
      | apply!(keyed::read_record::<CDAMP2>,301,3,70) => { |s| Record::CDAMP2(s) }
      | apply!(keyed::read_record::<CONM2>,1501,15,64) => { |s| Record::CONM2(s) }
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