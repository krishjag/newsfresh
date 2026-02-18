use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationType {
    Unknown = 0,
    Country = 1,
    UsState = 2,
    UsCity = 3,
    WorldCity = 4,
    WorldState = 5,
}

impl From<i32> for LocationType {
    fn from(val: i32) -> Self {
        match val {
            1 => Self::Country,
            2 => Self::UsState,
            3 => Self::UsCity,
            4 => Self::WorldCity,
            5 => Self::WorldState,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocationV1 {
    pub location_type: i32,
    pub full_name: String,
    pub country_code: String,
    pub adm1_code: String,
    pub latitude: f64,
    pub longitude: f64,
    pub feature_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnhancedLocation {
    pub location_type: i32,
    pub full_name: String,
    pub country_code: String,
    pub adm1_code: String,
    pub adm2_code: String,
    pub latitude: f64,
    pub longitude: f64,
    pub feature_id: String,
    pub char_offset: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_values_map_correctly() {
        assert_eq!(LocationType::from(1), LocationType::Country);
        assert_eq!(LocationType::from(2), LocationType::UsState);
        assert_eq!(LocationType::from(3), LocationType::UsCity);
        assert_eq!(LocationType::from(4), LocationType::WorldCity);
        assert_eq!(LocationType::from(5), LocationType::WorldState);
    }

    #[test]
    fn zero_maps_to_unknown() {
        assert_eq!(LocationType::from(0), LocationType::Unknown);
    }

    #[test]
    fn unknown_value_maps_to_unknown() {
        assert_eq!(LocationType::from(99), LocationType::Unknown);
        assert_eq!(LocationType::from(-1), LocationType::Unknown);
        assert_eq!(LocationType::from(6), LocationType::Unknown);
    }
}
