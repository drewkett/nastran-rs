#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    nastran::op2::parse_buffer_single(data);
});
