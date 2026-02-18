use crate::model::tone::Tone;
use super::delimiters::parse_f64;

pub fn parse_tone(field: &str) -> Option<Tone> {
    if field.is_empty() {
        return None;
    }
    let parts: Vec<&str> = field.split(',').collect();
    if parts.is_empty() {
        return None;
    }
    Some(Tone {
        tone: parse_f64(parts.first().unwrap_or(&"0")),
        positive_score: parse_f64(parts.get(1).unwrap_or(&"0")),
        negative_score: parse_f64(parts.get(2).unwrap_or(&"0")),
        polarity: parse_f64(parts.get(3).unwrap_or(&"0")),
        activity_ref_density: parse_f64(parts.get(4).unwrap_or(&"0")),
        self_group_ref_density: parse_f64(parts.get(5).unwrap_or(&"0")),
        word_count: parts.get(6).unwrap_or(&"0").parse::<i64>().unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tone_empty() {
        assert!(parse_tone("").is_none());
    }

    #[test]
    fn test_parse_tone_valid() {
        let tone = parse_tone("1.5,3.2,1.7,4.9,12.3,0.8,245").unwrap();
        assert!((tone.tone - 1.5).abs() < 1e-10);
        assert!((tone.positive_score - 3.2).abs() < 1e-10);
        assert!((tone.negative_score - 1.7).abs() < 1e-10);
        assert!((tone.polarity - 4.9).abs() < 1e-10);
        assert!((tone.activity_ref_density - 12.3).abs() < 1e-10);
        assert!((tone.self_group_ref_density - 0.8).abs() < 1e-10);
        assert_eq!(tone.word_count, 245);
    }

    #[test]
    fn test_parse_tone_partial_fields() {
        let tone = parse_tone("2.5").unwrap();
        assert!((tone.tone - 2.5).abs() < 1e-10);
        assert_eq!(tone.word_count, 0);
    }
}
