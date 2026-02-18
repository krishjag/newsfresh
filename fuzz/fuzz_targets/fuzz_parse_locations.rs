#![no_main]
use libfuzzer_sys::fuzz_target;

/// Fuzz the record parser with focus on location fields (V1 at index 9, V2 at index 10).
/// Locations use complex #-delimited blocks with ;-separated entries.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Build a minimal record with fuzzed location fields
        let mut fields = vec!["id"; 27];
        fields[1] = "20250101000000";
        fields[2] = "1";
        fields[9] = s;   // v1_locations
        fields[10] = s;  // v2_enhanced_locations
        let line = fields.join("\t");
        let _ = newsfresh::parse::parse_record(&line, 0);
    }
});
