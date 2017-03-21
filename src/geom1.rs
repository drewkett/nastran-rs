
use op2;
use keyed;
use nom::IResult;

#[derive(Debug)]
struct GRID {
    id: i32,
    cp: i32,
    x1: f32,
    x2: f32,
    x3: f32,
    cd: i32,
    ps: i32,
    seid: i32,
}

#[derive(Debug)]
struct CORD2R {
    id: i32,
    one: i32,
    two: i32,
    rid: i32,
    a1: f32,
    a2: f32,
    a3: f32,
    b1: f32,
    b2: f32,
    b3: f32,
    c1: f32,
    c2: f32,
    c3: f32,
}

pub type DataBlock<'a> = keyed::DataBlock<'a, Record<'a>>;

#[derive(Debug)]
enum Record<'a> {
    GRID(&'a [GRID]),
    CORD2R(&'a [CORD2R]),
    Unknown(keyed::UnknownRecord<'a>),
}

named!(read_record<Record>,
    alt!(
      apply!(keyed::read_record::<GRID>,4501,45,1) => { |s| Record::GRID(s) }
      | apply!(keyed::read_record::<CORD2R>,2101,21,8) => { |s| Record::CORD2R(s) }
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