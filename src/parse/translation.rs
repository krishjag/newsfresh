use crate::model::translation::TranslationInfo;

pub fn parse_translation_info(field: &str) -> Option<TranslationInfo> {
    if field.is_empty() {
        return None;
    }
    let mut source_language = String::new();
    let mut engine = String::new();

    for part in field.split(';') {
        let part = part.trim();
        if let Some(lang) = part.strip_prefix("srclc:") {
            source_language = lang.trim().to_string();
        } else if let Some(eng) = part.strip_prefix("eng:") {
            engine = eng.trim().to_string();
        }
    }

    if source_language.is_empty() && engine.is_empty() {
        None
    } else {
        Some(TranslationInfo {
            source_language,
            engine,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_translation_info_empty() {
        assert!(parse_translation_info("").is_none());
    }

    #[test]
    fn test_parse_translation_info_both_fields() {
        let result = parse_translation_info("srclc:ara;eng:1").unwrap();
        assert_eq!(result.source_language, "ara");
        assert_eq!(result.engine, "1");
    }

    #[test]
    fn test_parse_translation_info_only_srclc() {
        let result = parse_translation_info("srclc:fra").unwrap();
        assert_eq!(result.source_language, "fra");
        assert_eq!(result.engine, "");
    }

    #[test]
    fn test_parse_translation_info_only_eng() {
        let result = parse_translation_info("eng:1").unwrap();
        assert_eq!(result.source_language, "");
        assert_eq!(result.engine, "1");
    }

    #[test]
    fn test_parse_translation_info_no_recognized_fields() {
        assert!(parse_translation_info("unknown:value").is_none());
    }
}
