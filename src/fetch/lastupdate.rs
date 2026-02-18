use crate::error::NewsfreshError;

#[derive(Debug, Clone)]
pub struct LastUpdateEntry {
    pub size_bytes: u64,
    pub md5_hash: String,
    pub url: String,
}

pub fn parse_lastupdate(text: &str) -> Vec<LastUpdateEntry> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                return None;
            }
            Some(LastUpdateEntry {
                size_bytes: parts[0].parse().unwrap_or(0),
                md5_hash: parts[1].to_string(),
                url: parts[2].to_string(),
            })
        })
        .collect()
}

pub fn find_gkg_url(entries: &[LastUpdateEntry]) -> Result<String, NewsfreshError> {
    entries
        .iter()
        .find(|e| e.url.contains(".gkg.csv"))
        .map(|e| e.url.clone())
        .ok_or(NewsfreshError::NoGkgFile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lastupdate_valid() {
        let text = "12345 abc123hash http://data.gdeltproject.org/gdeltv2/20250217.export.csv.zip\n\
                    67890 def456hash http://data.gdeltproject.org/gdeltv2/20250217.gkg.csv.zip\n\
                    11111 ghi789hash http://data.gdeltproject.org/gdeltv2/20250217.mentions.csv.zip\n";
        let entries = parse_lastupdate(text);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].size_bytes, 12345);
        assert_eq!(
            entries[1].url,
            "http://data.gdeltproject.org/gdeltv2/20250217.gkg.csv.zip"
        );
    }

    #[test]
    fn test_parse_lastupdate_malformed_line() {
        let text = "only_one_field\n12345 hash url\n";
        let entries = parse_lastupdate(text);
        assert_eq!(entries.len(), 1); // malformed line skipped
    }

    #[test]
    fn test_find_gkg_url_found() {
        let entries = vec![
            LastUpdateEntry {
                size_bytes: 100,
                md5_hash: "abc".into(),
                url: "http://example.com/export.csv.zip".into(),
            },
            LastUpdateEntry {
                size_bytes: 200,
                md5_hash: "def".into(),
                url: "http://example.com/20250217.gkg.csv.zip".into(),
            },
        ];
        let url = find_gkg_url(&entries).unwrap();
        assert!(url.contains(".gkg.csv"));
    }

    #[test]
    fn test_find_gkg_url_not_found() {
        let entries = vec![LastUpdateEntry {
            size_bytes: 100,
            md5_hash: "abc".into(),
            url: "http://example.com/export.csv.zip".into(),
        }];
        assert!(find_gkg_url(&entries).is_err());
    }
}
