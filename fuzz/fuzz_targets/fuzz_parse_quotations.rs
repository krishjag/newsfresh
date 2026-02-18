#![no_main]
use libfuzzer_sys::fuzz_target;

/// Fuzz the record parser with focus on quotations (index 22), names (23),
/// amounts (24), and translation (25) — all in one target.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let mut fields = vec!["id"; 27];
        fields[1] = "20250101000000";
        fields[2] = "1";
        fields[22] = s;  // quotations
        fields[23] = s;  // all_names
        fields[24] = s;  // amounts
        fields[25] = s;  // translation_info
        let line = fields.join("\t");
        let _ = newsfresh::parse::parse_record(&line, 0);
    }
});
