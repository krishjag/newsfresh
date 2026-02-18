use std::collections::HashSet;
use std::io::Write;

use polars::prelude::*;

use crate::error::NewsfreshError;
use crate::model::GkgRecord;
use crate::search::fips;

pub struct AnalysisStats {
    pub total_records: usize,
    pub themes: Vec<FrequencyEntry>,
    pub countries: Vec<FrequencyEntry>,
    pub persons: Vec<FrequencyEntry>,
    pub organizations: Vec<FrequencyEntry>,
    pub sources: Vec<FrequencyEntry>,
    pub tone: Option<ToneStats>,
}

pub struct FrequencyEntry {
    pub name: String,
    pub count: u32,
    pub pct: f64,
}

pub struct ToneStats {
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub most_positive: ArticleRef,
    pub most_negative: ArticleRef,
}

pub struct ArticleRef {
    pub url: String,
    pub tone: f64,
}

pub fn compute_stats(
    records: &[(f32, &GkgRecord)],
    top_n: usize,
) -> Result<AnalysisStats, NewsfreshError> {
    // Extract flat vectors for each dimension
    let mut themes = Vec::new();
    let mut countries = Vec::new();
    let mut persons = Vec::new();
    let mut organizations = Vec::new();
    let mut sources = Vec::new();
    let mut tone_urls = Vec::new();
    let mut tone_values = Vec::new();

    for (_score, record) in records {
        // Themes: deduplicate per record, use display name
        let mut seen_themes = HashSet::new();
        for et in &record.v2_enhanced_themes {
            let display = display_theme(&et.theme);
            if seen_themes.insert(display.clone()) {
                themes.push(display);
            }
        }

        // Countries: deduplicate per record, display as "Name (CODE)"
        let mut seen_countries = HashSet::new();
        for loc in &record.v2_enhanced_locations {
            if !loc.country_code.is_empty() && seen_countries.insert(loc.country_code.clone()) {
                let name = fips::country_name(&loc.country_code)
                    .unwrap_or(&loc.country_code);
                countries.push(format!("{name} ({code})", code = loc.country_code));
            }
        }

        // Persons
        for p in &record.v1_persons {
            if !p.is_empty() {
                persons.push(p.clone());
            }
        }

        // Organizations
        for o in &record.v1_organizations {
            if !o.is_empty() {
                organizations.push(o.clone());
            }
        }

        // Source (one per record)
        if !record.source_common_name.is_empty() {
            sources.push(record.source_common_name.clone());
        }

        // Tone
        if let Some(ref tone) = record.tone {
            tone_urls.push(record.document_identifier.clone());
            tone_values.push(tone.tone);
        }
    }

    let tone_stats = compute_tone_stats(&tone_urls, &tone_values)?;

    Ok(AnalysisStats {
        total_records: records.len(),
        themes: compute_frequency(themes, top_n)?,
        countries: compute_frequency(countries, top_n)?,
        persons: compute_frequency(persons, top_n)?,
        organizations: compute_frequency(organizations, top_n)?,
        sources: compute_frequency(sources, top_n)?,
        tone: tone_stats,
    })
}

fn compute_frequency(
    values: Vec<String>,
    top_n: usize,
) -> Result<Vec<FrequencyEntry>, NewsfreshError> {
    if values.is_empty() {
        return Ok(Vec::new());
    }
    let total = values.len() as f64;
    let df = DataFrame::new(vec![Column::new("name".into(), &values)])?;

    let result = df
        .lazy()
        .group_by([col("name")])
        .agg([col("name").count().alias("count")])
        .sort(
            ["count"],
            SortMultipleOptions::default().with_order_descending(true),
        )
        .limit(top_n as u32)
        .collect()?;

    let names = result.column("name")?.str()?;
    let counts = result.column("count")?.u32()?;

    let mut entries = Vec::new();
    for i in 0..result.height() {
        let name = names.get(i).unwrap_or("").to_string();
        let count = counts.get(i).unwrap_or(0);
        entries.push(FrequencyEntry {
            name,
            count,
            pct: (count as f64 / total) * 100.0,
        });
    }
    Ok(entries)
}

fn compute_tone_stats(
    urls: &[String],
    tones: &[f64],
) -> Result<Option<ToneStats>, NewsfreshError> {
    if tones.is_empty() {
        return Ok(None);
    }

    let df = DataFrame::new(vec![
        Column::new("url".into(), urls),
        Column::new("tone".into(), tones),
    ])?;

    // Scalar aggregates
    let agg = df
        .clone()
        .lazy()
        .select([
            col("tone").mean().alias("mean"),
            col("tone").std(1).alias("std"),
            col("tone").min().alias("min"),
            col("tone").max().alias("max"),
        ])
        .collect()?;

    let mean = agg.column("mean")?.f64()?.get(0).unwrap_or(0.0);
    let std_dev = agg.column("std")?.f64()?.get(0).unwrap_or(0.0);
    let min = agg.column("min")?.f64()?.get(0).unwrap_or(0.0);
    let max = agg.column("max")?.f64()?.get(0).unwrap_or(0.0);

    // Most positive
    let most_pos = df
        .clone()
        .lazy()
        .sort(
            ["tone"],
            SortMultipleOptions::default().with_order_descending(true),
        )
        .limit(1)
        .collect()?;

    let pos_url = most_pos
        .column("url")?
        .str()?
        .get(0)
        .unwrap_or("")
        .to_string();
    let pos_tone = most_pos.column("tone")?.f64()?.get(0).unwrap_or(0.0);

    // Most negative
    let most_neg = df
        .lazy()
        .sort(
            ["tone"],
            SortMultipleOptions::default().with_order_descending(false),
        )
        .limit(1)
        .collect()?;

    let neg_url = most_neg
        .column("url")?
        .str()?
        .get(0)
        .unwrap_or("")
        .to_string();
    let neg_tone = most_neg.column("tone")?.f64()?.get(0).unwrap_or(0.0);

    Ok(Some(ToneStats {
        mean,
        std_dev,
        min,
        max,
        most_positive: ArticleRef {
            url: pos_url,
            tone: pos_tone,
        },
        most_negative: ArticleRef {
            url: neg_url,
            tone: neg_tone,
        },
    }))
}

pub fn print_stats(stats: &AnalysisStats, w: &mut dyn Write) -> Result<(), NewsfreshError> {
    writeln!(
        w,
        "\n=== GDELT Analysis Stats ({} records) ===",
        stats.total_records
    )?;

    print_frequency_table(w, "Top Themes", &stats.themes)?;
    print_frequency_table(w, "Top Countries", &stats.countries)?;

    if let Some(ref tone) = stats.tone {
        writeln!(w, "\n--- Tone ---")?;
        writeln!(
            w,
            "  Mean: {:.2}  Std: {:.2}  Range: [{:.2}, {:.2}]",
            tone.mean, tone.std_dev, tone.min, tone.max
        )?;
        writeln!(
            w,
            "  Most positive: [{:.2}] {}",
            tone.most_positive.tone, tone.most_positive.url
        )?;
        writeln!(
            w,
            "  Most negative: [{:.2}] {}",
            tone.most_negative.tone, tone.most_negative.url
        )?;
    }

    print_frequency_table(w, "Top Persons", &stats.persons)?;
    print_frequency_table(w, "Top Organizations", &stats.organizations)?;
    print_frequency_table(w, "Top Sources", &stats.sources)?;

    writeln!(w)?;
    Ok(())
}

fn print_frequency_table(
    w: &mut dyn Write,
    title: &str,
    entries: &[FrequencyEntry],
) -> Result<(), NewsfreshError> {
    if entries.is_empty() {
        return Ok(());
    }
    writeln!(w, "\n--- {title} ---")?;
    let max_name_len = entries.iter().map(|e| e.name.len()).max().unwrap_or(20);
    for (i, entry) in entries.iter().enumerate() {
        writeln!(
            w,
            "  {:>2}. {:<width$}  {:>4}  ({:.1}%)",
            i + 1,
            entry.name,
            entry.count,
            entry.pct,
            width = max_name_len
        )?;
    }
    Ok(())
}

fn display_theme(theme: &str) -> String {
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
        "MEDIA_",
    ];

    for prefix in PREFIXES {
        if let Some(rest) = theme.strip_prefix(prefix) {
            if !rest.is_empty() {
                return rest.replace('_', " ");
            }
        }
    }

    // WB_123_TOPIC → TOPIC
    if theme.starts_with("WB_") {
        if let Some(pos) = theme[3..].find('_') {
            let after = &theme[3 + pos + 1..];
            if !after.is_empty() {
                return after.replace('_', " ");
            }
        }
    }

    theme.replace('_', " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn make_test_record(source: &str, person: &str, tone_val: f64) -> GkgRecord {
        GkgRecord {
            gkg_record_id: "test-1".into(),
            date: 20250217120000,
            source_collection_id: 1,
            source_common_name: source.into(),
            document_identifier: format!("https://{source}/article"),
            v1_persons: vec![person.into()],
            v2_enhanced_persons: vec![],
            v1_organizations: vec!["congress".into()],
            v2_enhanced_organizations: vec![],
            v1_themes: vec![],
            v2_enhanced_themes: vec![EnhancedTheme {
                theme: "ELECTION".into(),
                char_offset: 0,
            }],
            v1_locations: vec![],
            v2_enhanced_locations: vec![EnhancedLocation {
                location_type: 1,
                full_name: "United States".into(),
                country_code: "US".into(),
                adm1_code: "US06".into(),
                adm2_code: "".into(),
                latitude: 38.0,
                longitude: -97.0,
                feature_id: "US".into(),
                char_offset: 0,
            }],
            tone: Some(Tone {
                tone: tone_val,
                positive_score: 2.0,
                negative_score: 3.5,
                polarity: 5.5,
                activity_ref_density: 10.0,
                self_group_ref_density: 0.5,
                word_count: 500,
            }),
            quotations: vec![],
            sharing_image: None,
            v1_counts: vec![],
            v21_counts: vec![],
            v21_enhanced_dates: vec![],
            gcam: vec![],
            related_images: vec![],
            social_image_embeds: vec![],
            social_video_embeds: vec![],
            all_names: vec![],
            amounts: vec![],
            translation_info: None,
            extras_xml: None,
        }
    }

    #[test]
    fn test_display_theme_with_prefix() {
        assert_eq!(display_theme("TAX_FNCACT_PRESIDENT"), "PRESIDENT");
        assert_eq!(
            display_theme("WB_696_PUBLIC_SECTOR_MANAGEMENT"),
            "PUBLIC SECTOR MANAGEMENT"
        );
        assert_eq!(display_theme("ELECTION"), "ELECTION");
        assert_eq!(display_theme("MEDIA_SOCIAL"), "SOCIAL");
    }

    #[test]
    fn test_compute_stats_small_dataset() {
        let r1 = make_test_record("cnn.com", "Alice Smith", -3.5);
        let r2 = make_test_record("bbc.co.uk", "Bob Jones", 1.2);
        let r3 = make_test_record("cnn.com", "Alice Smith", 5.0);

        let records: Vec<(f32, &GkgRecord)> =
            vec![(1.0, &r1), (0.8, &r2), (0.6, &r3)];

        let stats = compute_stats(&records, 5).expect("compute_stats should succeed");

        assert_eq!(stats.total_records, 3);
        assert!(!stats.themes.is_empty(), "themes should not be empty");
        assert!(!stats.persons.is_empty(), "persons should not be empty");
        assert!(!stats.sources.is_empty(), "sources should not be empty");

        let tone = stats.tone.as_ref().expect("tone stats should be present");
        // Mean of -3.5, 1.2, 5.0 is approximately 0.9
        assert!(tone.mean > -4.0 && tone.mean < 6.0, "mean should be reasonable");
        assert!(tone.min <= -3.5, "min should be <= -3.5");
        assert!(tone.max >= 5.0, "max should be >= 5.0");
    }

    #[test]
    fn test_compute_stats_empty() {
        let records: Vec<(f32, &GkgRecord)> = vec![];

        let stats = compute_stats(&records, 5).expect("compute_stats on empty should succeed");

        assert_eq!(stats.total_records, 0);
        assert!(stats.themes.is_empty());
        assert!(stats.persons.is_empty());
        assert!(stats.sources.is_empty());
        assert!(stats.countries.is_empty());
        assert!(stats.organizations.is_empty());
        assert!(stats.tone.is_none());
    }

    #[test]
    fn test_print_stats_output() {
        let stats = AnalysisStats {
            total_records: 42,
            themes: vec![FrequencyEntry {
                name: "CLIMATE CHANGE".into(),
                count: 10,
                pct: 23.8,
            }],
            countries: vec![FrequencyEntry {
                name: "United States (US)".into(),
                count: 30,
                pct: 71.4,
            }],
            persons: vec![FrequencyEntry {
                name: "John Doe".into(),
                count: 5,
                pct: 11.9,
            }],
            organizations: vec![FrequencyEntry {
                name: "United Nations".into(),
                count: 8,
                pct: 19.0,
            }],
            sources: vec![FrequencyEntry {
                name: "cnn.com".into(),
                count: 15,
                pct: 35.7,
            }],
            tone: Some(ToneStats {
                mean: -1.5,
                std_dev: 2.3,
                min: -8.0,
                max: 4.0,
                most_positive: ArticleRef {
                    url: "https://example.com/positive".into(),
                    tone: 4.0,
                },
                most_negative: ArticleRef {
                    url: "https://example.com/negative".into(),
                    tone: -8.0,
                },
            }),
        };

        let mut buf: Vec<u8> = Vec::new();
        print_stats(&stats, &mut buf).expect("print_stats should succeed");

        let output = String::from_utf8(buf).expect("output should be valid UTF-8");

        assert!(output.contains("GDELT Analysis Stats"), "should contain header");
        assert!(output.contains("42 records"), "should contain record count");
        assert!(output.contains("Top Themes"), "should contain Top Themes section");
        assert!(output.contains("CLIMATE CHANGE"), "should contain theme name");
        assert!(output.contains("Top Countries"), "should contain Top Countries section");
        assert!(output.contains("Tone"), "should contain Tone section");
        assert!(output.contains("Top Persons"), "should contain Top Persons section");
        assert!(output.contains("Top Organizations"), "should contain Top Organizations section");
        assert!(output.contains("Top Sources"), "should contain Top Sources section");
    }
}
