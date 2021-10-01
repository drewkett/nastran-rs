#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    let _ = nastran::bdf::v0::parser::parse_buffer(data);
});
