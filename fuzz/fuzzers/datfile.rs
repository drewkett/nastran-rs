#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate nastran;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    nastran::datfile::parse_buffer(data)
});
