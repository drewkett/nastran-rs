#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate nastran;

fuzz_target!(|data: &[u8]| {
    nastran::op2::parse_buffer(data);
});
