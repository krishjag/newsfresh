use thiserror::Error;

#[derive(Debug, Error)]
pub enum NewsfreshError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP extraction error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("Parse error at line {line}: {message}")]
    Parse { line: usize, message: String },

    #[error("Invalid date format: {0}")]
    InvalidDate(String),

    #[error("No GKG file found in lastupdate response")]
    NoGkgFile,

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[cfg(feature = "stats")]
    #[error("Polars error: {0}")]
    Polars(#[from] polars::error::PolarsError),

    #[error("{0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        let err = NewsfreshError::Parse {
            line: 42,
            message: "bad field".to_string(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("42"));
        assert!(msg.contains("bad field"));
    }

    #[test]
    fn test_no_gkg_file_display() {
        let err = NewsfreshError::NoGkgFile;
        let msg = format!("{err}");
        assert!(msg.contains("No GKG file"));
    }

    #[test]
    fn test_other_error_display() {
        let err = NewsfreshError::Other("custom error".to_string());
        assert_eq!(format!("{err}"), "custom error");
    }
}
