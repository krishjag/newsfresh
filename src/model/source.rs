use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceCollectionId {
    Web = 1,
    CitationOnly = 2,
    Core = 3,
    Dtic = 4,
    Jstor = 5,
    NonTextual = 6,
    Unknown = 0,
}

impl From<i32> for SourceCollectionId {
    fn from(val: i32) -> Self {
        match val {
            1 => Self::Web,
            2 => Self::CitationOnly,
            3 => Self::Core,
            4 => Self::Dtic,
            5 => Self::Jstor,
            6 => Self::NonTextual,
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_values_map_correctly() {
        assert_eq!(SourceCollectionId::from(1), SourceCollectionId::Web);
        assert_eq!(SourceCollectionId::from(2), SourceCollectionId::CitationOnly);
        assert_eq!(SourceCollectionId::from(3), SourceCollectionId::Core);
        assert_eq!(SourceCollectionId::from(4), SourceCollectionId::Dtic);
        assert_eq!(SourceCollectionId::from(5), SourceCollectionId::Jstor);
        assert_eq!(SourceCollectionId::from(6), SourceCollectionId::NonTextual);
    }

    #[test]
    fn zero_maps_to_unknown() {
        assert_eq!(SourceCollectionId::from(0), SourceCollectionId::Unknown);
    }

    #[test]
    fn unknown_value_maps_to_unknown() {
        assert_eq!(SourceCollectionId::from(99), SourceCollectionId::Unknown);
        assert_eq!(SourceCollectionId::from(-1), SourceCollectionId::Unknown);
        assert_eq!(SourceCollectionId::from(7), SourceCollectionId::Unknown);
    }
}
