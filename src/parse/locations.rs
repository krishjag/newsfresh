use super::delimiters::*;
use crate::model::location::{EnhancedLocation, LocationV1};

pub fn parse_locations_v1(field: &str) -> Vec<LocationV1> {
    if field.is_empty() {
        return Vec::new();
    }
    field
        .split(';')
        .filter(|s| !s.is_empty())
        .filter_map(|block| {
            let parts: Vec<&str> = block.split('#').collect();
            if parts.is_empty() {
                return None;
            }
            Some(LocationV1 {
                location_type: parse_i32(hash_field(&parts, 0)),
                full_name: hash_field(&parts, 1).to_string(),
                country_code: hash_field(&parts, 2).to_string(),
                adm1_code: hash_field(&parts, 3).to_string(),
                latitude: parse_f64(hash_field(&parts, 4)),
                longitude: parse_f64(hash_field(&parts, 5)),
                feature_id: hash_field(&parts, 6).to_string(),
            })
        })
        .collect()
}

pub fn parse_enhanced_locations(field: &str) -> Vec<EnhancedLocation> {
    if field.is_empty() {
        return Vec::new();
    }
    field
        .split(';')
        .filter(|s| !s.is_empty())
        .filter_map(|block| {
            let parts: Vec<&str> = block.split('#').collect();
            if parts.is_empty() {
                return None;
            }
            // V2Enhanced adds ADM2Code between ADM1Code and Latitude, plus CharOffset at end
            Some(EnhancedLocation {
                location_type: parse_i32(hash_field(&parts, 0)),
                full_name: hash_field(&parts, 1).to_string(),
                country_code: hash_field(&parts, 2).to_string(),
                adm1_code: hash_field(&parts, 3).to_string(),
                adm2_code: hash_field(&parts, 4).to_string(),
                latitude: parse_f64(hash_field(&parts, 5)),
                longitude: parse_f64(hash_field(&parts, 6)),
                feature_id: hash_field(&parts, 7).to_string(),
                char_offset: parse_i64(hash_field(&parts, 8)),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_locations_v1_empty() {
        let result = parse_locations_v1("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_locations_v1_single() {
        let result = parse_locations_v1(
            "4#Washington, District of Columbia, United States#US#USDC#38.8951#-77.0364#531871",
        );
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].location_type, 4);
        assert_eq!(
            result[0].full_name,
            "Washington, District of Columbia, United States"
        );
        assert_eq!(result[0].country_code, "US");
        assert_eq!(result[0].adm1_code, "USDC");
        assert!((result[0].latitude - 38.8951).abs() < 1e-4);
        assert!((result[0].longitude - (-77.0364)).abs() < 1e-4);
        assert_eq!(result[0].feature_id, "531871");
    }

    #[test]
    fn test_parse_enhanced_locations_empty() {
        let result = parse_enhanced_locations("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_enhanced_locations_single() {
        let result = parse_enhanced_locations(
            "4#Washington, District of Columbia, United States#US#USDC##38.8951#-77.0364#531871#120",
        );
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].location_type, 4);
        assert_eq!(
            result[0].full_name,
            "Washington, District of Columbia, United States"
        );
        assert_eq!(result[0].country_code, "US");
        assert_eq!(result[0].adm1_code, "USDC");
        assert_eq!(result[0].adm2_code, "");
        assert!((result[0].latitude - 38.8951).abs() < 1e-4);
        assert!((result[0].longitude - (-77.0364)).abs() < 1e-4);
        assert_eq!(result[0].feature_id, "531871");
        assert_eq!(result[0].char_offset, 120);
    }
}
