use crate::model::quotation::Quotation;
use super::delimiters::parse_i64;

pub fn parse_quotations(field: &str) -> Vec<Quotation> {
    if field.is_empty() {
        return Vec::new();
    }
    field
        .split('#')
        .filter(|s| !s.is_empty())
        .filter_map(|block| {
            let parts: Vec<&str> = block.split('|').collect();
            if parts.len() < 4 {
                return None;
            }
            Some(Quotation {
                offset: parse_i64(parts[0]),
                length: parse_i64(parts[1]),
                verb: parts[2].to_string(),
                quote: parts[3].to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_quotations_empty() {
        let result = parse_quotations("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_quotations_single() {
        let result = parse_quotations("120|45|said|we must act now to address climate change");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].offset, 120);
        assert_eq!(result[0].length, 45);
        assert_eq!(result[0].verb, "said");
        assert_eq!(result[0].quote, "we must act now to address climate change");
    }

    #[test]
    fn test_parse_quotations_multiple() {
        let result = parse_quotations(
            "120|45|said|we must act now#300|30|stated|the economy is growing"
        );
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].offset, 120);
        assert_eq!(result[0].verb, "said");
        assert_eq!(result[1].offset, 300);
        assert_eq!(result[1].verb, "stated");
        assert_eq!(result[1].quote, "the economy is growing");
    }
}
