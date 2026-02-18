use crate::model::count::{CountV1, CountV21};
use crate::model::location::LocationV1;
use super::delimiters::*;

pub fn parse_counts_v1(field: &str) -> Vec<CountV1> {
    if field.is_empty() {
        return Vec::new();
    }
    field
        .split(';')
        .filter(|s| !s.is_empty())
        .filter_map(|block| {
            let parts: Vec<&str> = block.split('#').collect();
            if parts.len() < 2 {
                return None;
            }
            Some(CountV1 {
                count_type: hash_field(&parts, 0).to_string(),
                count: parse_i64(hash_field(&parts, 1)),
                object_type: hash_field(&parts, 2).to_string(),
                location: LocationV1 {
                    location_type: parse_i32(hash_field(&parts, 3)),
                    full_name: hash_field(&parts, 4).to_string(),
                    country_code: hash_field(&parts, 5).to_string(),
                    adm1_code: hash_field(&parts, 6).to_string(),
                    latitude: parse_f64(hash_field(&parts, 7)),
                    longitude: parse_f64(hash_field(&parts, 8)),
                    feature_id: hash_field(&parts, 9).to_string(),
                },
            })
        })
        .collect()
}

pub fn parse_counts_v21(field: &str) -> Vec<CountV21> {
    if field.is_empty() {
        return Vec::new();
    }
    field
        .split(';')
        .filter(|s| !s.is_empty())
        .filter_map(|block| {
            let parts: Vec<&str> = block.split('#').collect();
            if parts.len() < 2 {
                return None;
            }
            Some(CountV21 {
                count_type: hash_field(&parts, 0).to_string(),
                count: parse_i64(hash_field(&parts, 1)),
                object_type: hash_field(&parts, 2).to_string(),
                location: LocationV1 {
                    location_type: parse_i32(hash_field(&parts, 3)),
                    full_name: hash_field(&parts, 4).to_string(),
                    country_code: hash_field(&parts, 5).to_string(),
                    adm1_code: hash_field(&parts, 6).to_string(),
                    latitude: parse_f64(hash_field(&parts, 7)),
                    longitude: parse_f64(hash_field(&parts, 8)),
                    feature_id: hash_field(&parts, 9).to_string(),
                },
                char_offset: parse_i64(hash_field(&parts, 10)),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_counts_v1_empty() {
        let result = parse_counts_v1("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_counts_v1_single() {
        let result = parse_counts_v1(
            "KILL#5#attackers#4#Baghdad, Baghdad, Iraq#IZ#IZ05#33.34#44.4#-1552751"
        );
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count_type, "KILL");
        assert_eq!(result[0].count, 5);
        assert_eq!(result[0].object_type, "attackers");
        assert_eq!(result[0].location.location_type, 4);
        assert_eq!(result[0].location.full_name, "Baghdad, Baghdad, Iraq");
        assert_eq!(result[0].location.country_code, "IZ");
        assert_eq!(result[0].location.feature_id, "-1552751");
    }

    #[test]
    fn test_parse_counts_v21_empty() {
        let result = parse_counts_v21("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_counts_v21_single() {
        let result = parse_counts_v21(
            "KILL#5#attackers#4#Baghdad, Baghdad, Iraq#IZ#IZ05#33.34#44.4#-1552751#300"
        );
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count_type, "KILL");
        assert_eq!(result[0].count, 5);
        assert_eq!(result[0].object_type, "attackers");
        assert_eq!(result[0].location.location_type, 4);
        assert_eq!(result[0].location.full_name, "Baghdad, Baghdad, Iraq");
        assert_eq!(result[0].char_offset, 300);
    }
}
