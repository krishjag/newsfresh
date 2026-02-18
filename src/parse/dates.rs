use crate::model::date::EnhancedDate;
use super::delimiters::{parse_i32, parse_i64};

pub fn parse_enhanced_dates(field: &str) -> Vec<EnhancedDate> {
    if field.is_empty() {
        return Vec::new();
    }
    field
        .split(';')
        .filter(|s| !s.is_empty())
        .filter_map(|block| {
            let parts: Vec<&str> = block.split(',').collect();
            if parts.is_empty() {
                return None;
            }
            Some(EnhancedDate {
                resolution: parse_i32(parts.first().unwrap_or(&"0")),
                month: parse_i32(parts.get(1).unwrap_or(&"0")),
                day: parse_i32(parts.get(2).unwrap_or(&"0")),
                year: parse_i32(parts.get(3).unwrap_or(&"0")),
                char_offset: parse_i64(parts.get(4).unwrap_or(&"0")),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_enhanced_dates_empty() {
        let result = parse_enhanced_dates("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_enhanced_dates_single() {
        let result = parse_enhanced_dates("4,3,15,2024,210");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].resolution, 4);
        assert_eq!(result[0].month, 3);
        assert_eq!(result[0].day, 15);
        assert_eq!(result[0].year, 2024);
        assert_eq!(result[0].char_offset, 210);
    }

    #[test]
    fn test_parse_enhanced_dates_multiple() {
        let result = parse_enhanced_dates("4,3,15,2024,210;3,6,0,2023,480");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].year, 2024);
        assert_eq!(result[0].day, 15);
        assert_eq!(result[1].year, 2023);
        assert_eq!(result[1].month, 6);
        assert_eq!(result[1].day, 0);
        assert_eq!(result[1].char_offset, 480);
    }
}
