use super::delimiters::parse_i64;
use crate::model::theme::EnhancedTheme;

pub fn parse_themes_v1(field: &str) -> Vec<String> {
    if field.is_empty() {
        return Vec::new();
    }
    field
        .split(';')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

pub fn parse_enhanced_themes(field: &str) -> Vec<EnhancedTheme> {
    if field.is_empty() {
        return Vec::new();
    }
    field
        .split(';')
        .filter(|s| !s.is_empty())
        .filter_map(|block| {
            let mut parts = block.splitn(2, ',');
            let theme = parts.next()?.to_string();
            let offset = parts.next().map(parse_i64).unwrap_or(0);
            Some(EnhancedTheme {
                theme,
                char_offset: offset,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_themes_v1_empty() {
        let result = parse_themes_v1("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_themes_v1_populated() {
        let result =
            parse_themes_v1("TAX_FNCACT_PRESIDENT;CRISISLEX_C03_WELLBEING;WB_2024_GOVERNANCE");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "TAX_FNCACT_PRESIDENT");
        assert_eq!(result[1], "CRISISLEX_C03_WELLBEING");
        assert_eq!(result[2], "WB_2024_GOVERNANCE");
    }

    #[test]
    fn test_parse_enhanced_themes_empty() {
        let result = parse_enhanced_themes("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_enhanced_themes_populated() {
        let result = parse_enhanced_themes("TAX_FNCACT_PRESIDENT,210;CRISISLEX_C03_WELLBEING,450");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].theme, "TAX_FNCACT_PRESIDENT");
        assert_eq!(result[0].char_offset, 210);
        assert_eq!(result[1].theme, "CRISISLEX_C03_WELLBEING");
        assert_eq!(result[1].char_offset, 450);
    }
}
