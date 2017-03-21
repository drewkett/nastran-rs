
use std::fmt;

use nom::IResult;

use op2;
use ident;

pub struct Ident {
    pub acode: i32,
    pub tcode: i32,
    pub datcod: i32,
    pub subcase: i32,
    pub var1: [u8; 12],
    pub rcode: i32,
    pub fcode: i32,
    pub numwde: i32,
    pub undef1: [i32; 2],
    pub acflag: i32,
    pub undef2: [i32; 3],
    pub rmssf: f32,
    pub undef3: [i32; 5],
    pub thermal: i32,
    pub undef4: [i32; 27],
    pub title: [u8; 128],
    pub subtitl: [u8; 128],
    pub label: [u8; 128],
}

pub struct Data {
    pub ekey: i32,
    pub etype: i32,
    pub data: [f32; 12],
}

pub type DataBlock<'a> = ident::DataBlockIdentPair<'a, Ident, Data>;

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let title = String::from_utf8_lossy(&self.title);
        let subtitle = String::from_utf8_lossy(&self.subtitl);
        let label = String::from_utf8_lossy(&self.label);
        write!(f,
               "OUG_IDENT[acode={},tcode={},title={},subtitle={},label={}]",
               self.acode,
               self.tcode,
               title,
               subtitle,
               label)
    }
}

pub fn read_datablock<'a>(input: &'a [u8],
                          start: op2::DataBlockStart<'a>)
                          -> IResult<&'a [u8], DataBlock<'a>> {
    let (input, header) = try_parse!(input, op2::read_datablock_header);
    let (input, record_pairs) = try_parse!(input,
                                           many0!(pair!(ident::read_ident::<Ident>,
                                                        ident::read_data::<Data>)));
    let (input, _) = try_parse!(input, op2::read_last_table_record);
    IResult::Done(input,
                  DataBlock {
                      name: start.name,
                      trailer: start.trailer,
                      record_type: start.record_type,
                      header: header,
                      record_pairs: record_pairs,
                  })
}
