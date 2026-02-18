/// Canonicalizes a GDELT theme code into human-readable text.
///
/// Strips known taxonomy prefixes and converts underscores to spaces.
/// Returns the original (with underscores→spaces) AND the stripped form
/// concatenated, so both match during search.
pub fn canonicalize_theme(theme: &str) -> String {
    let readable = theme.replace('_', " ");

    let stripped = strip_prefix(theme);
    if stripped != theme {
        let stripped_readable = stripped.replace('_', " ");
        format!("{readable} {stripped_readable}")
    } else {
        readable
    }
}

fn strip_prefix(theme: &str) -> &str {
    // Ordered longest-prefix-first to avoid partial matches
    const PREFIXES: &[&str] = &[
        "TAX_TERROR_GROUP_",
        "TAX_POLITICAL_PARTY_",
        "TAX_WORLDLANGUAGES_",
        "TAX_WORLDMAMMALS_",
        "TAX_WORLDBIRDS_",
        "TAX_WORLDREPTILES_",
        "TAX_WORLDFISH_",
        "TAX_ETHNICITY_",
        "TAX_FNCACT_",
        "CRISISLEX_",
        "EPU_CATS_",
        "EPU_POLICY_",
        "USPEC_POLITICS_",
        "USPEC_UNCERTAINTY",
        "MEDIA_",
    ];

    for prefix in PREFIXES {
        if let Some(rest) = theme.strip_prefix(prefix)
            && !rest.is_empty()
        {
            return rest;
        }
    }

    // World Bank codes: WB_123_TOPIC → TOPIC
    if theme.starts_with("WB_")
        && let Some(pos) = theme[3..].find('_')
    {
        let after_number = &theme[3 + pos + 1..];
        if !after_number.is_empty() {
            return after_number;
        }
    }

    theme
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terror_group() {
        let result = canonicalize_theme("TAX_TERROR_GROUP_BHARATIYA_JANATA_PARTY");
        assert!(result.contains("BHARATIYA JANATA PARTY"));
        assert!(result.contains("TAX TERROR GROUP"));
    }

    #[test]
    fn test_political_party() {
        let result = canonicalize_theme("TAX_POLITICAL_PARTY_BHARATIYA_JANATA_PARTY");
        assert!(result.contains("BHARATIYA JANATA PARTY"));
    }

    #[test]
    fn test_world_bank() {
        let result = canonicalize_theme("WB_696_PUBLIC_SECTOR_MANAGEMENT");
        assert!(result.contains("PUBLIC SECTOR MANAGEMENT"));
    }

    #[test]
    fn test_ethnicity() {
        let result = canonicalize_theme("TAX_ETHNICITY_TAMIL");
        assert!(result.contains("TAMIL"));
    }

    #[test]
    fn test_plain_theme() {
        let result = canonicalize_theme("ELECTION");
        assert_eq!(result, "ELECTION");
    }

    #[test]
    fn test_epu_policy() {
        let result = canonicalize_theme("EPU_POLICY_CONGRESS");
        assert!(result.contains("CONGRESS"));
    }

    #[test]
    fn test_media_prefix() {
        let result = canonicalize_theme("MEDIA_SOCIAL");
        assert!(result.contains("SOCIAL"));
    }

    #[test]
    fn test_uspec_politics() {
        let result = canonicalize_theme("USPEC_POLITICS_GENERAL1");
        assert!(result.contains("GENERAL1"));
    }

    #[test]
    fn test_empty_theme() {
        let result = canonicalize_theme("");
        assert_eq!(result, "");
    }
}
