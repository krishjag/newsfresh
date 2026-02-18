use crate::model::amount::AmountEntry;
use super::delimiters::*;

pub fn parse_amounts(field: &str) -> Vec<AmountEntry> {
    if field.is_empty() {
        return Vec::new();
    }
    field
        .split(';')
        .filter(|s| !s.is_empty())
        .filter_map(|block| {
            let parts: Vec<&str> = block.splitn(3, ',').collect();
            if parts.is_empty() {
                return None;
            }
            Some(AmountEntry {
                amount: parse_f64(parts.first().unwrap_or(&"0")),
                object: parts.get(1).unwrap_or(&"").to_string(),
                char_offset: parse_i64(parts.get(2).unwrap_or(&"0")),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_amounts_empty() {
        let result = parse_amounts("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_amounts_single() {
        let result = parse_amounts("1500000,dollars,210");
        assert_eq!(result.len(), 1);
        assert!((result[0].amount - 1_500_000.0).abs() < 1e-10);
        assert_eq!(result[0].object, "dollars");
        assert_eq!(result[0].char_offset, 210);
    }

    #[test]
    fn test_parse_amounts_multiple() {
        let result = parse_amounts("1500000,dollars,210;300,troops,450");
        assert_eq!(result.len(), 2);
        assert!((result[0].amount - 1_500_000.0).abs() < 1e-10);
        assert_eq!(result[0].object, "dollars");
        assert!((result[1].amount - 300.0).abs() < 1e-10);
        assert_eq!(result[1].object, "troops");
        assert_eq!(result[1].char_offset, 450);
    }
}
