use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::NewsfreshError;

pub fn extract_gkg_from_zip(zip_path: &Path, output_dir: &Path) -> Result<PathBuf, NewsfreshError> {
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        if name.ends_with(".csv") {
            let out_path = output_dir.join(&name);
            let mut out_file = std::fs::File::create(&out_path)?;
            std::io::copy(&mut entry, &mut out_file)?;
            return Ok(out_path);
        }
    }

    Err(NewsfreshError::Other("No CSV file found in ZIP".into()))
}

/// Read the GKG CSV content directly from a zip file without extracting to disk.
pub fn read_gkg_from_zip(zip_path: &Path) -> Result<String, NewsfreshError> {
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        if name.ends_with(".csv") {
            let mut content = String::new();
            entry.read_to_string(&mut content)?;
            return Ok(content);
        }
    }

    Err(NewsfreshError::Other("No CSV file found in ZIP".into()))
}
