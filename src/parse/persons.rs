use crate::model::person::EnhancedEntity;
use super::delimiters::parse_i64;

pub fn parse_persons_v1(field: &str) -> Vec<String> {
    if field.is_empty() {
        return Vec::new();
    }
    field
        .split(';')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

pub fn parse_enhanced_entities(field: &str) -> Vec<EnhancedEntity> {
    if field.is_empty() {
        return Vec::new();
    }
    field
        .split(';')
        .filter(|s| !s.is_empty())
        .filter_map(|block| {
            // Name comes first, then comma, then char offset
            let comma_pos = block.rfind(',')?;
            let name = block[..comma_pos].to_string();
            let offset = parse_i64(&block[comma_pos + 1..]);
            if name.is_empty() {
                return None;
            }
            Some(EnhancedEntity {
                name,
                char_offset: offset,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_persons_v1_empty() {
        let result = parse_persons_v1("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_persons_v1_populated() {
        let result = parse_persons_v1("Barack Obama;Joe Biden;Angela Merkel");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "Barack Obama");
        assert_eq!(result[1], "Joe Biden");
        assert_eq!(result[2], "Angela Merkel");
    }

    #[test]
    fn test_parse_enhanced_entities_empty() {
        let result = parse_enhanced_entities("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_enhanced_entities_populated() {
        let result = parse_enhanced_entities("Barack Obama,120;Joe Biden,340");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "Barack Obama");
        assert_eq!(result[0].char_offset, 120);
        assert_eq!(result[1].name, "Joe Biden");
        assert_eq!(result[1].char_offset, 340);
    }

    #[test]
    fn test_parse_enhanced_entities_name_with_comma() {
        // rfind ensures last comma is used as delimiter, so names with commas work
        let result = parse_enhanced_entities("Biden, Joe,250");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Biden, Joe");
        assert_eq!(result[0].char_offset, 250);
    }
}
