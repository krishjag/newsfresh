use super::delimiters::parse_i64;
use crate::model::name::NameEntry;

pub fn parse_names(field: &str) -> Vec<NameEntry> {
    if field.is_empty() {
        return Vec::new();
    }
    field
        .split(';')
        .filter(|s| !s.is_empty())
        .filter_map(|block| {
            let comma_pos = block.rfind(',')?;
            let name = block[..comma_pos].to_string();
            let offset = parse_i64(&block[comma_pos + 1..]);
            if name.is_empty() {
                return None;
            }
            Some(NameEntry {
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
    fn test_parse_names_empty() {
        let result = parse_names("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_names_single() {
        let result = parse_names("United Nations,350");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "United Nations");
        assert_eq!(result[0].char_offset, 350);
    }

    #[test]
    fn test_parse_names_multiple() {
        let result = parse_names("United Nations,350;European Union,580");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "United Nations");
        assert_eq!(result[0].char_offset, 350);
        assert_eq!(result[1].name, "European Union");
        assert_eq!(result[1].char_offset, 580);
    }

    #[test]
    fn test_parse_names_name_with_comma() {
        // rfind ensures last comma is the delimiter
        let result = parse_names("Biden, Joe,120");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Biden, Joe");
        assert_eq!(result[0].char_offset, 120);
    }
}
