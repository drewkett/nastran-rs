
use op2;
use nom::IResult;

#[derive(Debug)]
pub struct DataBlock<'a> {
    pub name: &'a str,
    pub trailer: op2::DataBlockTrailer<'a>,
    pub record_type: op2::DataBlockType,
    pub header: &'a [u8],
    pub records: Vec<&'a [u8]>,
}

named!(read_table_record,do_parse!(
  apply!(op2::read_nastran_known_i32,0) >>
  data : call!(op2::read_nastran_data) >>
  call!(op2::read_nastran_eor) >>
  (data)
));

pub fn read_datablock<'a>(input: &'a [u8],
                      start: op2::DataBlockStart<'a>)
                      -> IResult<&'a [u8], DataBlock<'a>> {
    let (input, header) = try_parse!(input, op2::read_datablock_header);
    let (input, records) = try_parse!(input, many0!(read_table_record));
    let (input, _) = try_parse!(input, op2::read_last_table_record);
    IResult::Done(input,
                  DataBlock {
                      name: start.name,
                      trailer: start.trailer,
                      record_type: start.record_type,
                      header: header,
                      records: records,
                  })
}