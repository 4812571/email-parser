#![no_main]

use libfuzzer_sys::fuzz_target;
use email_parser::prelude::*;

fuzz_target!(|data: &[u8]| {
    let _ = Email::parse(data);
});