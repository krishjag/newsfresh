use super::delimiters::parse_f64;
use crate::model::gcam::GcamEntry;

pub fn parse_gcam(field: &str) -> Vec<GcamEntry> {
    if field.is_empty() {
        return Vec::new();
    }
    field
        .split(',')
        .filter(|s| !s.is_empty())
        .filter_map(|pair| {
            let mut kv = pair.splitn(2, ':');
            let dimension = kv.next()?.to_string();
            let value = kv.next().map(parse_f64).unwrap_or(0.0);
            Some(GcamEntry { dimension, value })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gcam_empty() {
        let result = parse_gcam("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_gcam_single() {
        let result = parse_gcam("wc:18.34");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].dimension, "wc");
        assert!((result[0].value - 18.34).abs() < 1e-10);
    }

    #[test]
    fn test_parse_gcam_multiple() {
        let result = parse_gcam("wc:18.34,c12.1:5.2,v21.1:3.14159");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].dimension, "wc");
        assert!((result[0].value - 18.34).abs() < 1e-10);
        assert_eq!(result[1].dimension, "c12.1");
        assert!((result[1].value - 5.2).abs() < 1e-10);
        assert_eq!(result[2].dimension, "v21.1");
        assert!((result[2].value - 3.14159).abs() < 1e-5);
    }
}
