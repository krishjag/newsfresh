use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::io::AsyncWriteExt;

use crate::error::NewsfreshError;

fn build_client() -> Result<reqwest::Client, NewsfreshError> {
    Ok(reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?)
}

pub async fn fetch_text(url: &str) -> Result<String, NewsfreshError> {
    let client = build_client()?;
    let resp = client.get(url).send().await?.error_for_status()?;
    Ok(resp.text().await?)
}

pub async fn download_file(url: &str, dest: &Path) -> Result<PathBuf, NewsfreshError> {
    let client = build_client()?;
    let resp = client.get(url).send().await?.error_for_status()?;

    let total_size = resp.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::with_template(
            "{msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec})",
        )
        .unwrap()
        .progress_chars("=> "),
    );
    pb.set_message("Downloading");

    let mut file = tokio::fs::File::create(dest).await?;
    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }

    file.flush().await?;
    pb.finish_with_message("Downloaded");

    Ok(dest.to_path_buf())
}

pub fn lastupdate_url(translation: bool) -> &'static str {
    if translation {
        "http://data.gdeltproject.org/gdeltv2/lastupdate-translation.txt"
    } else {
        "http://data.gdeltproject.org/gdeltv2/lastupdate.txt"
    }
}

pub fn historical_url(date: &str) -> String {
    format!(
        "http://data.gdeltproject.org/gdeltv2/{date}.gkg.csv.zip"
    )
}
