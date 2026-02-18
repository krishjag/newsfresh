#![no_main]
use libfuzzer_sys::fuzz_target;

/// Fuzz the record parser with focus on tone field (field index 15).
/// Constructs a minimal 27-field record with the fuzzed data in the tone position.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Build a minimal record with fuzzed tone field at position 15
        let mut fields = vec!["id"; 27];
        fields[1] = "20250101000000"; // date
        fields[2] = "1";              // source_collection_id
        fields[15] = s;               // tone field
        let line = fields.join("\t");
        let _ = newsfresh::parse::parse_record(&line, 0);
    }
});
