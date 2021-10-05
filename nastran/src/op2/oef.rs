use std::fmt;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Ident {
    pub acode: i32,
    pub tcode: i32,
    pub eltype: i32,
    pub subcase: i32,
    pub var1: [u8; 12],
    pub dloadid: i32,
    pub fcode: i32,
    pub numwde: i32,
    pub ocode: i32,
    pub pid: i32,
    pub undef1: i32,
    pub q4cstr: i32,
    pub plsloc: i32,
    pub undef2: i32,
    pub rmssf: f32,
    pub undef3: [i32; 5],
    pub thermal: i32,
    pub undef4: [i32; 27],
    pub title: [u8; 128],
    pub subtitl: [u8; 128],
    pub label: [u8; 128],
}

impl fmt::Debug for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use bstr::ByteSlice;
        f.debug_struct("Ident")
            .field("acode", &self.acode)
            .field("tcode", &self.tcode)
            .field("eltype", &self.eltype)
            .field("subcase", &self.subcase)
            .field("var1", &self.var1)
            .field("dloadid", &self.dloadid)
            .field("fcode", &self.fcode)
            .field("numwde", &self.numwde)
            .field("ocode", &self.ocode)
            .field("pid", &self.pid)
            .field("undef1", &self.undef1)
            .field("q4cstr", &self.q4cstr)
            .field("plsloc", &self.plsloc)
            .field("undef2", &self.undef2)
            .field("rmssf", &self.rmssf)
            .field("undef3", &self.undef3)
            .field("thermal", &self.thermal)
            .field("undef4", &self.undef4)
            .field("title", &self.title.as_bstr())
            .field("subtitl", &self.subtitl.as_bstr())
            .field("label", &self.label.as_bstr())
            .finish()
    }
}

// SAFETY All zeros is a valid value
unsafe impl bytemuck::Zeroable for Ident {}
// SAFETY Any value is valid, there is no padding, the underlying type is Pod and its repr(C)
unsafe impl bytemuck::Pod for Ident {}

pub struct CROD {
    var: i32,
    af: f32,
    trq: f32,
}
