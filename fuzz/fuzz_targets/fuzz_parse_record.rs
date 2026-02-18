#![no_main]
use libfuzzer_sys::fuzz_target;

/// Fuzz the main GKG record parser with arbitrary tab-delimited input.
/// This exercises ALL sub-parsers: tone, themes, persons, locations, counts,
/// quotations, amounts, names, dates, gcam, and translation.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // parse_record should never panic regardless of input
        let _ = newsfresh::parse::parse_record(s, 0);
    }
});
