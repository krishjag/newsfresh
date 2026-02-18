/// Split a field by the given delimiter, filtering out empty trailing entries.
pub fn split_blocks(input: &str, delimiter: char) -> Vec<&str> {
    if input.is_empty() {
        return Vec::new();
    }
    input.split(delimiter).filter(|s| !s.is_empty()).collect()
}

/// Split a semicolon-delimited field into a `Vec<String>`.
pub fn split_semicolon_list(input: &str) -> Vec<String> {
    if input.is_empty() {
        return Vec::new();
    }
    input
        .split(';')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Return Some(String) if non-empty, None otherwise.
pub fn non_empty(input: &str) -> Option<String> {
    if input.is_empty() {
        None
    } else {
        Some(input.to_string())
    }
}

/// Parse a field from a #-delimited block by index, returning empty string if missing.
pub fn hash_field<'a>(parts: &[&'a str], index: usize) -> &'a str {
    parts.get(index).copied().unwrap_or("")
}

/// Parse a float from a string, defaulting to 0.0.
pub fn parse_f64(s: &str) -> f64 {
    s.parse::<f64>().unwrap_or(0.0)
}

/// Parse an i64 from a string, defaulting to 0.
pub fn parse_i64(s: &str) -> i64 {
    s.parse::<i64>().unwrap_or(0)
}

/// Parse an i32 from a string, defaulting to 0.
pub fn parse_i32(s: &str) -> i32 {
    s.parse::<i32>().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_blocks_empty() {
        assert!(split_blocks("", ';').is_empty());
    }

    #[test]
    fn test_split_blocks_multiple_with_trailing() {
        assert_eq!(split_blocks("a;b;c;", ';'), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_split_semicolon_list_empty() {
        assert!(split_semicolon_list("").is_empty());
    }

    #[test]
    fn test_split_semicolon_list_populated() {
        assert_eq!(split_semicolon_list("alpha;beta"), vec!["alpha", "beta"]);
    }

    #[test]
    fn test_non_empty() {
        assert_eq!(non_empty(""), None);
        assert_eq!(non_empty("hello"), Some("hello".to_string()));
    }

    #[test]
    fn test_hash_field() {
        let parts = vec!["a", "b", "c"];
        assert_eq!(hash_field(&parts, 1), "b");
        assert_eq!(hash_field(&parts, 5), "");
    }

    #[test]
    fn test_parse_f64() {
        assert!((parse_f64("3.14") - 3.14).abs() < 1e-10);
        assert!((parse_f64("abc")).abs() < 1e-10);
        assert!((parse_f64("")).abs() < 1e-10);
    }

    #[test]
    fn test_parse_i64() {
        assert_eq!(parse_i64("42"), 42);
        assert_eq!(parse_i64(""), 0);
    }

    #[test]
    fn test_parse_i32() {
        assert_eq!(parse_i32("100"), 100);
        assert_eq!(parse_i32("xyz"), 0);
    }
}
