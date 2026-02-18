#![no_main]
use libfuzzer_sys::fuzz_target;
use newsfresh::parse::delimiters;

/// Fuzz all public delimiter/parsing utility functions.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // None of these should ever panic
        let _ = delimiters::split_blocks(s, ';');
        let _ = delimiters::split_blocks(s, '#');
        let _ = delimiters::split_blocks(s, ',');
        let _ = delimiters::split_semicolon_list(s);
        let _ = delimiters::non_empty(s);
        let _ = delimiters::parse_f64(s);
        let _ = delimiters::parse_i64(s);
        let _ = delimiters::parse_i32(s);

        // Also test hash_field with fuzzed parts
        let parts: Vec<&str> = s.split('#').collect();
        for i in 0..parts.len().min(10) {
            let _ = delimiters::hash_field(&parts, i);
        }
    }
});
