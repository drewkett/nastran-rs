use std::fmt;
use crate::op2::prelude::*;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Ident<P: Precision> {
    acode: P::Int,
    tcode: P::Int,
    eltype: P::Int,
    subcase: P::Int,
    var1: [P::Word; 3],
    dloadid: P::Int,
    fcode: P::Int,
    numwde: P::Int,
    ocode: P::Int,
    pid: P::Int,
    undef1: P::Int,
    q4cstr: P::Int,
    plsloc: P::Int,
    undef2: P::Int,
    rmssf: P::Float,
    undef3: [P::Int; 5],
    thermal: P::Int,
    undef4: [P::Int; 27],
    title: [P::Word; 32],
    subtitl: [P::Word; 32],
    label: [P::Word; 32],
}

#[derive(Debug)]
pub enum OefMetaData<P:Precision> {
    Sort1Statics {
        load_id: P::Int
    }
}

impl <P: Precision> Ident<P> {
    fn device_code(&self) -> u8 {
        debug_assert!(self.acode > <P as Precision>::zero_int());
        let acode: i64 = self.acode.into();
        (acode % 10) as u8
    }

    fn approach_code(&self) -> u32 {
        let acode:i64 = self.acode.into();
        debug_assert!(acode > 0);
        debug_assert!(acode < (i32::MAX as i64));
        let acode = acode as u32;
        acode / 10
    }

    fn table_code(&self) -> i32 {
        let tcode:i64 = self.tcode.into();
        debug_assert!(tcode > 0);
        debug_assert!(tcode < (i32::MAX as i64));
        tcode as i32
    }

    fn eltype(&self) -> i32 {
        let eltype:i64 = self.eltype.into();
        debug_assert!(eltype > 0);
        debug_assert!(eltype < (i32::MAX as i64));
        eltype as i32
    }

    pub fn metadata(&self) -> OefMetaData<P> {
        match fun1::<P>(self.tcode) {
            OneOrTwo::One => match self.approach_code() {
                1 => OefMetaData::Sort1Statics {
                    load_id: P::word_to_int(self.var1[0])
                },
                _ => unimplemented!()
            }
            OneOrTwo::Two => {unimplemented!()}
        }
    }
}

impl <P: Precision> fmt::Debug for Ident<P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
            .field("title", &(&self.title[..]).debug_words())
            .field("subtitl", &(&self.subtitl[..]).debug_words())
            .field("label", &(&self.label[..]).debug_words())
            .finish()
    }
}

// SAFETY All zeros is a valid value
unsafe impl <P:Precision> bytemuck::Zeroable for Ident<P> {}
// SAFETY Any value is valid, there is no padding, the underlying type is Pod and its repr(C)
unsafe impl <P:Precision> bytemuck::Pod for Ident<P> {}

//pub struct CROD {
//    var: i32,
//    af: f32,
//    trq: f32,
//}
