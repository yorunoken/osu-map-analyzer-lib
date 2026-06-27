use std::{
    collections::{HashMap, HashSet},
    fmt,
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};

use super::{AnalysisError, AnalysisOptions, BeatmapAnalysis, BeatmapAnalyzer, GameMod, MapTag};

/// Supported human-assigned labels for ML datasets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ManualLabel {
    Aim,
    AimControl,
    Stream,
    Burst,
    Speed,
    Stamina,
    FingerControl,
    RhythmComplex,
    Reading,
    Tech,
    Slider,
    Precision,
    Farm,
    AimFarm,
    StreamFarm,
    Consistency,
    Awkward,
    Comfortable,
    AimSlop,
}

/// Machine-generated features. This row intentionally contains no manual labels.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureRow {
    pub file: String,
    pub beatmap_id: Option<i32>,
    pub beatmap_set_id: Option<i32>,
    pub artist: String,
    pub title: String,
    pub creator: String,
    pub version: String,
    #[serde(default)]
    pub mods: Vec<GameMod>,
    #[serde(default = "default_rate")]
    pub rate: f64,
    pub star_rating_nomod: Option<f64>,
    pub star_rating_adjusted: Option<f64>,
    pub feature_names: Vec<String>,
    pub feature_values: Vec<f32>,
    pub deterministic_tags: Vec<MapTag>,
}

/// Small, human-editable label row.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LabelTemplateRow {
    pub beatmap_id: Option<i32>,
    pub beatmap_set_id: Option<i32>,
    pub file: String,
    pub artist: String,
    pub title: String,
    pub creator: String,
    pub version: String,
    #[serde(default)]
    pub mods: Vec<GameMod>,
    #[serde(default = "default_rate")]
    pub rate: f64,
    pub star_rating_nomod: Option<f64>,
    pub star_rating_adjusted: Option<f64>,
    pub deterministic_tags: Vec<MapTag>,
    #[serde(default)]
    pub manual_labels: Vec<ManualLabel>,
}

/// Final row produced by merging generated features with human labels.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TrainingDatasetRow {
    pub beatmap_id: Option<i32>,
    pub beatmap_set_id: Option<i32>,
    pub file: String,
    pub artist: String,
    pub title: String,
    pub creator: String,
    pub version: String,
    #[serde(default)]
    pub mods: Vec<GameMod>,
    #[serde(default = "default_rate")]
    pub rate: f64,
    pub star_rating_nomod: Option<f64>,
    pub star_rating_adjusted: Option<f64>,
    pub feature_names: Vec<String>,
    pub feature_values: Vec<f32>,
    pub deterministic_tags: Vec<MapTag>,
    pub manual_labels: Vec<ManualLabel>,
}

impl FeatureRow {
    pub fn from_analysis(path: &Path, analysis: &BeatmapAnalysis) -> Self {
        let features = analysis.to_feature_vector();
        Self {
            file: path.to_string_lossy().into_owned(),
            beatmap_id: analysis.metadata.beatmap_id,
            beatmap_set_id: analysis.metadata.beatmap_set_id,
            artist: analysis.metadata.artist.clone(),
            title: analysis.metadata.title.clone(),
            creator: analysis.metadata.creator.clone(),
            version: analysis.metadata.version.clone(),
            mods: analysis.mods.clone(),
            rate: analysis.rate,
            star_rating_nomod: analysis.star_rating_nomod,
            star_rating_adjusted: analysis.star_rating_adjusted,
            feature_names: features.names,
            feature_values: features.values,
            deterministic_tags: analysis.tags.clone(),
        }
    }

    pub fn label_template(&self) -> LabelTemplateRow {
        LabelTemplateRow {
            beatmap_id: self.beatmap_id,
            beatmap_set_id: self.beatmap_set_id,
            file: self.file.clone(),
            artist: self.artist.clone(),
            title: self.title.clone(),
            creator: self.creator.clone(),
            version: self.version.clone(),
            mods: self.mods.clone(),
            rate: self.rate,
            star_rating_nomod: self.star_rating_nomod,
            star_rating_adjusted: self.star_rating_adjusted,
            deterministic_tags: self.deterministic_tags.clone(),
            manual_labels: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum DatasetError {
    Io(io::Error),
    Analyze {
        path: PathBuf,
        source: AnalysisError,
    },
    Json(serde_json::Error),
    InvalidFeatureLine {
        line: usize,
        source: serde_json::Error,
    },
    InvalidLabelLine {
        line: usize,
        source: serde_json::Error,
    },
    FeatureLengthMismatch {
        file: String,
        names: usize,
        values: usize,
    },
    DuplicateFeatureName {
        file: String,
        name: String,
    },
    NonFiniteFeature {
        file: String,
        index: usize,
    },
    InvalidRate {
        dataset: &'static str,
        file: String,
        rate: f64,
    },
    NonFiniteStarRating {
        file: String,
        field: &'static str,
    },
    InconsistentFeatures {
        file: String,
    },
    DuplicateMapKey {
        dataset: &'static str,
        key: String,
    },
    MissingFeatureRow {
        key: String,
    },
}

impl fmt::Display for DatasetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "dataset I/O failed: {error}"),
            Self::Analyze { path, source } => {
                write!(formatter, "failed to analyze {}: {source}", path.display())
            }
            Self::Json(error) => write!(formatter, "dataset JSON failed: {error}"),
            Self::InvalidFeatureLine { line, source } => {
                write!(formatter, "invalid feature JSONL at line {line}: {source}")
            }
            Self::InvalidLabelLine { line, source } => {
                write!(formatter, "invalid label JSONL at line {line}: {source}")
            }
            Self::FeatureLengthMismatch {
                file,
                names,
                values,
            } => write!(
                formatter,
                "feature names and values differ for {file}: {names} names, {values} values"
            ),
            Self::DuplicateFeatureName { file, name } => {
                write!(formatter, "duplicate feature name `{name}` in {file}")
            }
            Self::NonFiniteFeature { file, index } => {
                write!(
                    formatter,
                    "non-finite feature value at index {index} in {file}"
                )
            }
            Self::InvalidRate {
                dataset,
                file,
                rate,
            } => write!(
                formatter,
                "invalid playback rate {rate} for {file} in {dataset}; rate must be finite and positive"
            ),
            Self::NonFiniteStarRating { file, field } => {
                write!(formatter, "non-finite {field} in {file}")
            }
            Self::InconsistentFeatures { file } => {
                write!(
                    formatter,
                    "feature schema for {file} differs from the first row"
                )
            }
            Self::DuplicateMapKey { dataset, key } => {
                write!(formatter, "duplicate map key {key} in {dataset}")
            }
            Self::MissingFeatureRow { key } => {
                write!(formatter, "label row {key} has no matching feature row")
            }
        }
    }
}

impl std::error::Error for DatasetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Analyze { source, .. } => Some(source),
            Self::Json(error)
            | Self::InvalidFeatureLine { source: error, .. }
            | Self::InvalidLabelLine { source: error, .. } => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for DatasetError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for DatasetError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub fn analyze_dataset_paths(paths: &[PathBuf]) -> Result<Vec<FeatureRow>, DatasetError> {
    analyze_dataset_paths_with_options(paths, AnalysisOptions::default())
}

pub fn analyze_dataset_paths_with_options(
    paths: &[PathBuf],
    options: AnalysisOptions,
) -> Result<Vec<FeatureRow>, DatasetError> {
    paths
        .iter()
        .map(|path| {
            let analysis = BeatmapAnalyzer::from_path(path)
                .and_then(|analyzer| analyzer.analyze(options.clone()))
                .map_err(|source| DatasetError::Analyze {
                    path: path.clone(),
                    source,
                })?;
            Ok(FeatureRow::from_analysis(path, &analysis))
        })
        .collect()
}

pub fn write_feature_rows_jsonl(
    writer: &mut impl Write,
    rows: &[FeatureRow],
) -> Result<(), DatasetError> {
    write_jsonl(writer, rows)
}

pub fn write_label_templates_jsonl(
    writer: &mut impl Write,
    rows: &[LabelTemplateRow],
) -> Result<(), DatasetError> {
    write_jsonl(writer, rows)
}

pub fn write_training_dataset_jsonl(
    writer: &mut impl Write,
    rows: &[TrainingDatasetRow],
) -> Result<(), DatasetError> {
    write_jsonl(writer, rows)
}

pub fn write_feature_rows_csv(
    writer: &mut impl Write,
    rows: &[FeatureRow],
) -> Result<(), DatasetError> {
    let Some(first) = rows.first() else {
        return Ok(());
    };

    let mut header = vec![
        "file".to_owned(),
        "beatmap_id".to_owned(),
        "beatmap_set_id".to_owned(),
        "artist".to_owned(),
        "title".to_owned(),
        "creator".to_owned(),
        "version".to_owned(),
        "mods".to_owned(),
        "rate".to_owned(),
        "star_rating_nomod".to_owned(),
        "star_rating_adjusted".to_owned(),
        "deterministic_tags".to_owned(),
    ];
    header.extend(first.feature_names.iter().cloned());
    write_csv_record(writer, header.iter().map(String::as_str))?;

    for row in rows {
        validate_feature_row(row)?;
        if row.feature_names != first.feature_names {
            return Err(DatasetError::InconsistentFeatures {
                file: row.file.clone(),
            });
        }
        let tags = serde_json::to_string(&row.deterministic_tags)?;
        let mods = serde_json::to_string(&row.mods)?;
        let beatmap_id = optional_id(row.beatmap_id);
        let beatmap_set_id = optional_id(row.beatmap_set_id);
        let rate = row.rate.to_string();
        let star_rating_nomod = optional_float(row.star_rating_nomod);
        let star_rating_adjusted = optional_float(row.star_rating_adjusted);
        let feature_values: Vec<_> = row.feature_values.iter().map(ToString::to_string).collect();
        let mut fields = vec![
            row.file.as_str(),
            beatmap_id.as_str(),
            beatmap_set_id.as_str(),
            row.artist.as_str(),
            row.title.as_str(),
            row.creator.as_str(),
            row.version.as_str(),
            mods.as_str(),
            rate.as_str(),
            star_rating_nomod.as_str(),
            star_rating_adjusted.as_str(),
            tags.as_str(),
        ];
        fields.extend(feature_values.iter().map(String::as_str));
        write_csv_record(writer, fields)?;
    }
    Ok(())
}

pub fn parse_feature_rows(reader: impl BufRead) -> Result<Vec<FeatureRow>, DatasetError> {
    reader
        .lines()
        .enumerate()
        .filter_map(non_empty_line)
        .map(|(line_number, line)| {
            let row: FeatureRow = serde_json::from_str(&line?).map_err(|source| {
                DatasetError::InvalidFeatureLine {
                    line: line_number,
                    source,
                }
            })?;
            validate_feature_row(&row)?;
            Ok(row)
        })
        .collect()
}

pub fn parse_label_templates(reader: impl BufRead) -> Result<Vec<LabelTemplateRow>, DatasetError> {
    reader
        .lines()
        .enumerate()
        .filter_map(non_empty_line)
        .map(|(line_number, line)| {
            serde_json::from_str(&line?).map_err(|source| DatasetError::InvalidLabelLine {
                line: line_number,
                source,
            })
        })
        .collect()
}

pub fn merge_dataset(
    features: &[FeatureRow],
    labels: &[LabelTemplateRow],
) -> Result<Vec<TrainingDatasetRow>, DatasetError> {
    let mut feature_keys = HashMap::new();
    let expected_schema = features.first().map(|row| row.feature_names.as_slice());
    for row in features {
        validate_feature_row(row)?;
        if expected_schema.is_some_and(|schema| row.feature_names != schema) {
            return Err(DatasetError::InconsistentFeatures {
                file: row.file.clone(),
            });
        }
        let key = MatchKey::for_row(row.beatmap_id, &row.file, &row.mods, row.rate);
        if feature_keys.insert(key.clone(), ()).is_some() {
            return Err(DatasetError::DuplicateMapKey {
                dataset: "features",
                key: key.to_string(),
            });
        }
    }

    let mut labels_by_key = HashMap::new();
    for row in labels {
        validate_rate("labels", &row.file, row.rate)?;
        let key = MatchKey::for_row(row.beatmap_id, &row.file, &row.mods, row.rate);
        if labels_by_key.insert(key.clone(), row).is_some() {
            return Err(DatasetError::DuplicateMapKey {
                dataset: "labels",
                key: key.to_string(),
            });
        }
        if !feature_keys.contains_key(&key) {
            return Err(DatasetError::MissingFeatureRow {
                key: key.to_string(),
            });
        }
    }

    features
        .iter()
        .map(|feature| {
            let key = MatchKey::for_row(
                feature.beatmap_id,
                &feature.file,
                &feature.mods,
                feature.rate,
            );
            let manual_labels = labels_by_key
                .get(&key)
                .map_or_else(Vec::new, |row| row.manual_labels.clone());
            Ok(TrainingDatasetRow {
                beatmap_id: feature.beatmap_id,
                beatmap_set_id: feature.beatmap_set_id,
                file: feature.file.clone(),
                artist: feature.artist.clone(),
                title: feature.title.clone(),
                creator: feature.creator.clone(),
                version: feature.version.clone(),
                mods: feature.mods.clone(),
                rate: feature.rate,
                star_rating_nomod: feature.star_rating_nomod,
                star_rating_adjusted: feature.star_rating_adjusted,
                feature_names: feature.feature_names.clone(),
                feature_values: feature.feature_values.clone(),
                deterministic_tags: feature.deterministic_tags.clone(),
                manual_labels,
            })
        })
        .collect()
}

pub fn merge_dataset_jsonl(
    feature_reader: impl BufRead,
    label_reader: impl BufRead,
) -> Result<Vec<TrainingDatasetRow>, DatasetError> {
    let features = parse_feature_rows(feature_reader)?;
    let labels = parse_label_templates(label_reader)?;
    merge_dataset(&features, &labels)
}

fn validate_feature_row(row: &FeatureRow) -> Result<(), DatasetError> {
    validate_rate("features", &row.file, row.rate)?;
    if row.feature_names.len() != row.feature_values.len() {
        return Err(DatasetError::FeatureLengthMismatch {
            file: row.file.clone(),
            names: row.feature_names.len(),
            values: row.feature_values.len(),
        });
    }
    let mut names = HashSet::new();
    for name in &row.feature_names {
        if !names.insert(name) {
            return Err(DatasetError::DuplicateFeatureName {
                file: row.file.clone(),
                name: name.clone(),
            });
        }
    }
    if let Some(index) = row
        .feature_values
        .iter()
        .position(|value| !value.is_finite())
    {
        return Err(DatasetError::NonFiniteFeature {
            file: row.file.clone(),
            index,
        });
    }
    for (field, value) in [
        ("star_rating_nomod", row.star_rating_nomod),
        ("star_rating_adjusted", row.star_rating_adjusted),
    ] {
        if value.is_some_and(|value| !value.is_finite()) {
            return Err(DatasetError::NonFiniteStarRating {
                file: row.file.clone(),
                field,
            });
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MatchKey {
    map: MapIdentity,
    mods: Vec<GameMod>,
    rate_bits: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MapIdentity {
    BeatmapId(i32),
    File(String),
}

impl MatchKey {
    fn for_row(beatmap_id: Option<i32>, file: &str, mods: &[GameMod], rate: f64) -> Self {
        let map = match beatmap_id.filter(|id| *id > 0) {
            Some(id) => MapIdentity::BeatmapId(id),
            None => MapIdentity::File(file.to_owned()),
        };
        let mut mods = mods.to_vec();
        mods.sort_unstable();
        mods.dedup();

        Self {
            map,
            mods,
            rate_bits: rate.to_bits(),
        }
    }
}

impl fmt::Display for MatchKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.map {
            MapIdentity::BeatmapId(id) => write!(formatter, "beatmap_id {id}")?,
            MapIdentity::File(file) => write!(formatter, "file `{file}`")?,
        }
        let mods = if self.mods.is_empty() {
            "NM".to_owned()
        } else {
            self.mods.iter().map(ToString::to_string).collect()
        };
        write!(
            formatter,
            " with mods {mods} at rate {}",
            f64::from_bits(self.rate_bits)
        )
    }
}

fn validate_rate(dataset: &'static str, file: &str, rate: f64) -> Result<(), DatasetError> {
    if rate.is_finite() && rate > 0.0 {
        Ok(())
    } else {
        Err(DatasetError::InvalidRate {
            dataset,
            file: file.to_owned(),
            rate,
        })
    }
}

fn non_empty_line(
    (index, line): (usize, io::Result<String>),
) -> Option<(usize, io::Result<String>)> {
    match line {
        Ok(line) if line.trim().is_empty() => None,
        value => Some((index + 1, value)),
    }
}

fn write_jsonl<T: serde::Serialize>(
    writer: &mut impl Write,
    rows: &[T],
) -> Result<(), DatasetError> {
    for row in rows {
        serde_json::to_writer(&mut *writer, row)?;
        writer.write_all(b"\n")?;
    }
    Ok(())
}

fn optional_id(value: Option<i32>) -> String {
    value
        .filter(|id| *id > 0)
        .map_or_else(String::new, |id| id.to_string())
}

fn optional_float(value: Option<f64>) -> String {
    value.map_or_else(String::new, |value| value.to_string())
}

fn default_rate() -> f64 {
    1.0
}

fn write_csv_record<'a>(
    writer: &mut impl Write,
    fields: impl IntoIterator<Item = &'a str>,
) -> io::Result<()> {
    let mut first = true;
    for field in fields {
        if !first {
            writer.write_all(b",")?;
        }
        first = false;
        if field.contains([',', '"', '\n', '\r']) {
            writer.write_all(b"\"")?;
            writer.write_all(field.replace('"', "\"\"").as_bytes())?;
            writer.write_all(b"\"")?;
        } else {
            writer.write_all(field.as_bytes())?;
        }
    }
    writer.write_all(b"\n")
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, io::Cursor};

    use super::*;

    fn fixture_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("maps/map.osu")
    }

    fn feature_row() -> FeatureRow {
        analyze_dataset_paths(&[fixture_path()]).unwrap().remove(0)
    }

    fn feature_row_with(mods: Vec<GameMod>, rate: Option<f64>) -> FeatureRow {
        analyze_dataset_paths_with_options(
            &[fixture_path()],
            AnalysisOptions {
                mods,
                rate,
                ..AnalysisOptions::default()
            },
        )
        .unwrap()
        .remove(0)
    }

    #[test]
    fn manual_labels_round_trip_with_singular_names() {
        let labels = [
            ManualLabel::Aim,
            ManualLabel::AimControl,
            ManualLabel::Stream,
            ManualLabel::Burst,
            ManualLabel::Speed,
            ManualLabel::Stamina,
            ManualLabel::FingerControl,
            ManualLabel::RhythmComplex,
            ManualLabel::Reading,
            ManualLabel::Tech,
            ManualLabel::Slider,
            ManualLabel::Precision,
            ManualLabel::Farm,
            ManualLabel::AimFarm,
            ManualLabel::StreamFarm,
            ManualLabel::Consistency,
            ManualLabel::Awkward,
            ManualLabel::Comfortable,
            ManualLabel::AimSlop,
        ];

        for label in labels {
            let json = serde_json::to_string(&label).unwrap();
            assert!(!json.contains(' '));
            assert_eq!(serde_json::from_str::<ManualLabel>(&json).unwrap(), label);
        }
    }

    #[test]
    fn unknown_manual_label_reports_a_clear_error() {
        let mut label_json = serde_json::to_string(&feature_row().label_template()).unwrap();
        label_json = label_json.replace(
            "\"manual_labels\":[]",
            "\"manual_labels\":[\"UnknownLabel\"]",
        );

        let error = merge_dataset_jsonl(Cursor::new(""), Cursor::new(label_json)).unwrap_err();
        let message = error.to_string();
        assert!(
            message.contains("invalid label JSONL at line 1"),
            "{message}"
        );
        assert!(
            message.contains("unknown variant `UnknownLabel`"),
            "{message}"
        );
    }

    #[test]
    fn feature_export_has_valid_unique_names_and_no_manual_labels() {
        let row = feature_row();
        let names: HashSet<_> = row.feature_names.iter().collect();
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(row.feature_names.len(), row.feature_values.len());
        assert_eq!(names.len(), row.feature_names.len());
        assert!(row.feature_values.iter().all(|value| value.is_finite()));
        assert!(json.get("manual_labels").is_none());
        assert_eq!(json["mods"], serde_json::json!([]));
        assert_eq!(json["rate"], 1.0);
        assert!(json["star_rating_nomod"].is_number());
        assert!(json["star_rating_adjusted"].is_number());
        for removed in [
            "avg_bpm",
            "aim_pressure_mean",
            "aim_pressure_peak",
            "speed_pressure_mean",
            "speed_pressure_peak",
            "slider_pressure_mean",
            "slider_pressure_peak",
            "slider_velocity_changes",
            "avg_nps",
            "avg_spacing",
            "average_jump_distance",
            "stamina_pressure",
            "rhythm_variety",
            "rhythm_burstiness",
            "repeated_interval_patterns",
            "average_stream_bpm",
            "average_slider_duration",
            "slider_complexity",
            "inherited_timing_points",
        ] {
            assert!(!row.feature_names.iter().any(|name| name == removed));
        }
        for retained in [
            "star_rating_available",
            "star_rating_nomod",
            "star_rating_adjusted",
            "rate",
            "average_bpm",
            "aim_pressure_score_mean",
            "aim_pressure_score_peak",
            "speed_pressure_score_mean",
            "speed_pressure_score_peak",
            "slider_pressure_score_mean",
            "slider_pressure_score_peak",
            "slider_velocity_change_count",
            "notes_per_second_mean",
            "jump_distance_mean",
            "stamina_pressure_score",
            "rhythm_variety_ratio",
            "rhythm_burstiness_score",
            "repeated_interval_pattern_count",
            "stream_bpm_mean",
            "slider_duration_mean_ms",
            "slider_complexity_score",
            "inherited_timing_point_count",
        ] {
            assert!(row.feature_names.iter().any(|name| name == retained));
        }
    }

    #[test]
    fn feature_export_is_deterministic() {
        let first = feature_row();
        let second = feature_row();

        assert_eq!(first.feature_names, second.feature_names);
        assert_eq!(first.feature_values.len(), second.feature_values.len());
        for (left, right) in first.feature_values.iter().zip(&second.feature_values) {
            assert!((left - right).abs() <= 1e-6, "{left} != {right}");
        }
    }

    #[test]
    fn label_template_contains_set_id_and_starts_empty() {
        let feature = feature_row();
        let template = feature.label_template();

        assert_eq!(template.beatmap_set_id, feature.beatmap_set_id);
        assert_eq!(template.mods, feature.mods);
        assert_eq!(template.rate, feature.rate);
        assert_eq!(template.star_rating_nomod, feature.star_rating_nomod);
        assert_eq!(template.star_rating_adjusted, feature.star_rating_adjusted);
        assert!(template.manual_labels.is_empty());
    }

    #[test]
    fn default_and_mod_rates_are_resolved() {
        assert_eq!(feature_row().rate, 1.0);
        assert_eq!(feature_row_with(vec![GameMod::DT], None).rate, 1.5);
        assert_eq!(feature_row_with(vec![GameMod::NC], None).rate, 1.5);
        assert_eq!(feature_row_with(vec![GameMod::HT], None).rate, 0.75);
        let overridden = feature_row_with(vec![GameMod::DT], Some(1.25));
        assert_eq!(overridden.rate, 1.25);
        assert_eq!(overridden.mods, [GameMod::DT]);
    }

    #[test]
    fn invalid_and_conflicting_rates_return_clear_errors() {
        let analyzer = BeatmapAnalyzer::from_path(fixture_path()).unwrap();
        let invalid = analyzer
            .analyze(AnalysisOptions {
                rate: Some(0.0),
                ..AnalysisOptions::default()
            })
            .unwrap_err();
        assert!(invalid.to_string().contains("finite and positive"));

        let conflicting = analyzer
            .analyze(AnalysisOptions {
                mods: vec![GameMod::DT, GameMod::HT],
                ..AnalysisOptions::default()
            })
            .unwrap_err();
        assert!(conflicting
            .to_string()
            .contains("conflicting playback rates"));
    }

    #[test]
    fn rate_scales_time_metrics_but_not_jump_distance() {
        let analyzer = BeatmapAnalyzer::from_path(fixture_path()).unwrap();
        let nomod = analyzer.analyze(AnalysisOptions::default()).unwrap();
        let dt = analyzer
            .analyze(AnalysisOptions {
                mods: vec![GameMod::DT],
                ..AnalysisOptions::default()
            })
            .unwrap();

        assert!((dt.timing.main_bpm - nomod.timing.main_bpm * 1.5).abs() <= 1e-6);
        assert!((dt.general.total_length - nomod.general.total_length / 1.5).abs() <= 1e-6);
        assert!((dt.objects.notes_per_second - nomod.objects.notes_per_second * 1.5).abs() <= 1e-6);
        assert!((dt.aim.average_jump_distance - nomod.aim.average_jump_distance).abs() <= 1e-9);
    }

    #[test]
    fn mod_rate_and_star_metadata_round_trip_through_json() {
        let row = feature_row_with(vec![GameMod::HD, GameMod::DT], None);
        let json = serde_json::to_string(&row).unwrap();
        let parsed: FeatureRow = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.file, row.file);
        assert_eq!(parsed.feature_names, row.feature_names);
        assert_eq!(parsed.feature_values, row.feature_values);
        assert_eq!(parsed.deterministic_tags, row.deterministic_tags);
        assert_eq!(parsed.mods, [GameMod::DT, GameMod::HD]);
        assert_eq!(parsed.rate, 1.5);
        assert!(
            (parsed.star_rating_nomod.unwrap() - row.star_rating_nomod.unwrap()).abs() <= 1e-12
        );
        assert!(
            (parsed.star_rating_adjusted.unwrap() - row.star_rating_adjusted.unwrap()).abs()
                <= 1e-12
        );
    }

    #[test]
    fn missing_manual_labels_field_defaults_to_empty() {
        let mut json = serde_json::to_value(feature_row().label_template()).unwrap();
        json.as_object_mut().unwrap().remove("manual_labels");

        let labels = parse_label_templates(Cursor::new(json.to_string())).unwrap();

        assert_eq!(labels.len(), 1);
        assert!(labels[0].manual_labels.is_empty());
    }

    #[test]
    fn merge_combines_labels_and_preserves_deterministic_tags() {
        let feature = feature_row();
        let deterministic_tags = feature.deterministic_tags.clone();
        let mut label = feature.label_template();
        label.manual_labels = vec![
            ManualLabel::Stream,
            ManualLabel::Stamina,
            ManualLabel::Speed,
        ];

        let merged = merge_dataset(&[feature], &[label]).unwrap();

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].deterministic_tags, deterministic_tags);
        assert_eq!(
            merged[0].manual_labels,
            [
                ManualLabel::Stream,
                ManualLabel::Stamina,
                ManualLabel::Speed
            ]
        );
    }

    #[test]
    fn merge_uses_file_when_beatmap_id_is_missing() {
        let mut feature = feature_row();
        feature.beatmap_id = None;
        let mut label = feature.label_template();
        label.beatmap_id = Some(0);
        label.manual_labels = vec![ManualLabel::Aim];

        let merged = merge_dataset(&[feature], &[label]).unwrap();

        assert_eq!(merged[0].manual_labels, [ManualLabel::Aim]);
    }

    #[test]
    fn merge_keeps_unlabeled_features_with_empty_labels() {
        let merged = merge_dataset(&[feature_row()], &[]).unwrap();

        assert_eq!(merged.len(), 1);
        assert!(merged[0].manual_labels.is_empty());
    }

    #[test]
    fn merge_key_distinguishes_nomod_and_dt_for_the_same_map() {
        let nomod = feature_row();
        let dt = feature_row_with(vec![GameMod::DT], None);
        assert_eq!(nomod.beatmap_id, dt.beatmap_id);
        let mut nm_label = nomod.label_template();
        nm_label.manual_labels = vec![ManualLabel::Aim];
        let mut dt_label = dt.label_template();
        dt_label.manual_labels = vec![ManualLabel::Speed];

        let merged = merge_dataset(&[nomod, dt], &[nm_label, dt_label]).unwrap();

        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].manual_labels, [ManualLabel::Aim]);
        assert_eq!(merged[1].manual_labels, [ManualLabel::Speed]);
        assert_eq!(merged[1].mods, [GameMod::DT]);
        assert_eq!(merged[1].rate, 1.5);
    }

    #[test]
    fn merge_errors_when_label_has_no_feature_row() {
        let feature = feature_row();
        let mut label = feature.label_template();
        label.beatmap_id = Some(999_999_999);

        let error = merge_dataset(&[feature], &[label]).unwrap_err();

        assert!(error.to_string().contains("no matching feature row"));
    }

    #[test]
    fn merge_errors_on_duplicate_beatmap_ids() {
        let first = feature_row();
        let mut second = first.clone();
        second.file = "another.osu".to_owned();

        let error = merge_dataset(&[first, second], &[]).unwrap_err();
        let message = error.to_string();

        assert!(
            message.contains("duplicate map key beatmap_id"),
            "{message}"
        );
        assert!(message.contains("features"), "{message}");
    }

    #[test]
    fn merge_validates_feature_lengths_and_finite_values() {
        let mut mismatched = feature_row();
        mismatched.feature_values.pop();
        assert!(merge_dataset(&[mismatched], &[])
            .unwrap_err()
            .to_string()
            .contains("names and values differ"));

        let mut non_finite = feature_row();
        non_finite.feature_values[0] = f32::INFINITY;
        assert!(merge_dataset(&[non_finite], &[])
            .unwrap_err()
            .to_string()
            .contains("non-finite feature value"));
    }
}
