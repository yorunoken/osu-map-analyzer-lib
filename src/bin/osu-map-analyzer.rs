use std::{
    env,
    error::Error,
    ffi::{OsStr, OsString},
    io,
    path::{Path, PathBuf},
};

#[cfg(feature = "serialize")]
use std::fs;
#[cfg(feature = "serialize")]
use std::io::BufReader;

#[cfg(feature = "serialize")]
use osu_map_analyzer::{
    analyze_dataset_paths_with_options, merge_dataset_jsonl, write_feature_rows_csv,
    write_feature_rows_jsonl, write_label_templates_jsonl, write_training_dataset_jsonl,
    FeatureRow,
};
use osu_map_analyzer::{AnalysisOptions, BeatmapAnalysis, BeatmapAnalyzer, GameMod};

#[derive(Clone, Copy, PartialEq, Eq)]
enum OutputMode {
    Compact,
    Details,
    Json,
}

enum Command {
    Analyze {
        path: PathBuf,
        mode: OutputMode,
        options: AnalysisOptions,
    },
    ExportFeatures {
        inputs: Vec<PathBuf>,
        output: PathBuf,
        options: AnalysisOptions,
    },
    ExportLabelTemplate {
        inputs: Vec<PathBuf>,
        output: PathBuf,
        options: AnalysisOptions,
    },
    MergeDataset {
        features: PathBuf,
        labels: PathBuf,
        output: PathBuf,
    },
    Help,
}

fn main() -> Result<(), Box<dyn Error>> {
    match parse_args()? {
        Command::Analyze {
            path,
            mode,
            options,
        } => analyze_one(path, mode, options),
        Command::ExportFeatures {
            inputs,
            output,
            options,
        } => export_features(&inputs, &output, options),
        Command::ExportLabelTemplate {
            inputs,
            output,
            options,
        } => export_labels(&inputs, &output, options),
        Command::MergeDataset {
            features,
            labels,
            output,
        } => merge_files(&features, &labels, &output),
        Command::Help => {
            print_help();
            Ok(())
        }
    }
}

fn analyze_one(
    path: PathBuf,
    mode: OutputMode,
    options: AnalysisOptions,
) -> Result<(), Box<dyn Error>> {
    let analysis = BeatmapAnalyzer::from_path(path)?.analyze(options)?;

    match mode {
        OutputMode::Compact => println!("{}", analysis.compact_report()),
        OutputMode::Details => println!("{}", analysis.detailed_report()),
        OutputMode::Json => print_json(&analysis)?,
    }

    let validation = analysis.validate();
    eprintln!(
        "Validation: {} passed, {} warnings, {} errors",
        validation.passed.len(),
        validation.warnings.len(),
        validation.errors.len()
    );
    for warning in &validation.warnings {
        eprintln!("warning: {warning}");
    }
    if !validation.is_valid() {
        for error in &validation.errors {
            eprintln!("error: {error}");
        }
        return Err(
            io::Error::new(io::ErrorKind::InvalidData, "analysis validation failed").into(),
        );
    }
    Ok(())
}

#[cfg(feature = "serialize")]
fn export_features(
    inputs: &[PathBuf],
    output: &Path,
    options: AnalysisOptions,
) -> Result<(), Box<dyn Error>> {
    let paths = collect_map_paths(inputs)?;
    let rows = analyze_dataset_paths_with_options(&paths, options)?;
    warn_missing_stars(&rows);
    let mut file = fs::File::create(output)?;
    match output.extension().and_then(|extension| extension.to_str()) {
        Some(extension) if extension.eq_ignore_ascii_case("jsonl") => {
            write_feature_rows_jsonl(&mut file, &rows)?;
        }
        Some(extension) if extension.eq_ignore_ascii_case("csv") => {
            write_feature_rows_csv(&mut file, &rows)?;
        }
        _ => return Err(invalid_input("feature output must end in .jsonl or .csv")),
    }
    eprintln!(
        "Exported {} feature row(s) to {}",
        rows.len(),
        output.display()
    );
    Ok(())
}

#[cfg(not(feature = "serialize"))]
fn export_features(_: &[PathBuf], _: &Path, _: AnalysisOptions) -> Result<(), Box<dyn Error>> {
    Err(invalid_input(
        "export-features requires the serialize feature",
    ))
}

#[cfg(feature = "serialize")]
fn export_labels(
    inputs: &[PathBuf],
    output: &Path,
    options: AnalysisOptions,
) -> Result<(), Box<dyn Error>> {
    if !output
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("jsonl"))
    {
        return Err(invalid_input("label template output must end in .jsonl"));
    }
    let paths = collect_map_paths(inputs)?;
    let rows = analyze_dataset_paths_with_options(&paths, options)?;
    warn_missing_stars(&rows);
    let templates: Vec<_> = rows.iter().map(|row| row.label_template()).collect();
    let mut file = fs::File::create(output)?;
    write_label_templates_jsonl(&mut file, &templates)?;
    eprintln!(
        "Exported {} label template row(s) to {}",
        templates.len(),
        output.display()
    );
    Ok(())
}

#[cfg(not(feature = "serialize"))]
fn export_labels(_: &[PathBuf], _: &Path, _: AnalysisOptions) -> Result<(), Box<dyn Error>> {
    Err(invalid_input(
        "export-label-template requires the serialize feature",
    ))
}

#[cfg(feature = "serialize")]
fn merge_files(features: &Path, labels: &Path, output: &Path) -> Result<(), Box<dyn Error>> {
    if !output
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("jsonl"))
    {
        return Err(invalid_input("merged dataset output must end in .jsonl"));
    }
    let feature_file = BufReader::new(fs::File::open(features)?);
    let label_file = BufReader::new(fs::File::open(labels)?);
    let rows = merge_dataset_jsonl(feature_file, label_file)?;
    let mut output_file = fs::File::create(output)?;
    write_training_dataset_jsonl(&mut output_file, &rows)?;
    eprintln!(
        "Merged {} training row(s) into {}",
        rows.len(),
        output.display()
    );
    Ok(())
}

#[cfg(not(feature = "serialize"))]
fn merge_files(_: &Path, _: &Path, _: &Path) -> Result<(), Box<dyn Error>> {
    Err(invalid_input(
        "merge-dataset requires the serialize feature",
    ))
}

#[cfg(feature = "serialize")]
fn warn_missing_stars(rows: &[FeatureRow]) {
    for row in rows {
        if row.star_rating_nomod.is_none() || row.star_rating_adjusted.is_none() {
            eprintln!("warning: star rating unavailable for {}", row.file);
        }
    }
}

#[cfg(feature = "serialize")]
fn collect_map_paths(inputs: &[PathBuf]) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut paths = Vec::new();
    for input in inputs {
        collect_input(input, &mut paths, true)?;
    }
    paths.sort();
    paths.dedup();
    if paths.is_empty() {
        return Err(invalid_input("no .osu files found in the supplied inputs"));
    }
    Ok(paths)
}

#[cfg(feature = "serialize")]
fn collect_input(
    path: &Path,
    paths: &mut Vec<PathBuf>,
    require_osu_file: bool,
) -> Result<(), Box<dyn Error>> {
    if path.is_file() {
        if is_osu_file(path) {
            paths.push(path.to_owned());
            return Ok(());
        }
        return if require_osu_file {
            Err(invalid_input(&format!(
                "input file is not an .osu file: {}",
                path.display()
            )))
        } else {
            Ok(())
        };
    }
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            collect_input(&entry?.path(), paths, false)?;
        }
        return Ok(());
    }
    Err(invalid_input(&format!(
        "input path does not exist: {}",
        path.display()
    )))
}

#[cfg(feature = "serialize")]
fn is_osu_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("osu"))
}

fn parse_args() -> Result<Command, Box<dyn Error>> {
    let arguments: Vec<_> = env::args_os().skip(1).collect();
    match arguments.first().and_then(|argument| argument.to_str()) {
        Some("export-features") => parse_export_args(&arguments[1..], false),
        Some("export-label-template") => parse_export_args(&arguments[1..], true),
        Some("merge-dataset") => parse_merge_args(&arguments[1..]),
        Some("--help" | "-h") => Ok(Command::Help),
        _ => parse_analyze_args(&arguments),
    }
}

fn parse_merge_args(arguments: &[OsString]) -> Result<Command, Box<dyn Error>> {
    let mut features = None;
    let mut labels = None;
    let mut output = None;
    let mut index = 0;
    while index < arguments.len() {
        let target = match arguments[index].to_str() {
            Some("--features") => &mut features,
            Some("--labels") => &mut labels,
            Some("--out") => &mut output,
            Some("--help" | "-h") => return Ok(Command::Help),
            _ => {
                return Err(invalid_input(&format!(
                    "unknown merge option: {}",
                    arguments[index].to_string_lossy()
                )))
            }
        };
        index += 1;
        let value = arguments
            .get(index)
            .ok_or_else(|| invalid_input("merge option requires a path"))?;
        if target.replace(PathBuf::from(value)).is_some() {
            return Err(invalid_input("merge options may only be provided once"));
        }
        index += 1;
    }

    Ok(Command::MergeDataset {
        features: features.ok_or_else(|| invalid_input("--features is required"))?,
        labels: labels.ok_or_else(|| invalid_input("--labels is required"))?,
        output: output.ok_or_else(|| invalid_input("--out is required"))?,
    })
}

fn parse_export_args(arguments: &[OsString], labels: bool) -> Result<Command, Box<dyn Error>> {
    let mut inputs = Vec::new();
    let mut output = None;
    let mut options = AnalysisOptions::default();
    let mut index = 0;
    while index < arguments.len() {
        if arguments[index] == "--out" {
            index += 1;
            let value = arguments
                .get(index)
                .ok_or_else(|| invalid_input("--out requires a path"))?;
            if output.replace(PathBuf::from(value)).is_some() {
                return Err(invalid_input("--out may only be provided once"));
            }
        } else if arguments[index] == "--mods" {
            index += 1;
            let value = arguments
                .get(index)
                .ok_or_else(|| invalid_input("--mods requires a value"))?;
            parse_mods(value, &mut options.mods)?;
        } else if arguments[index] == "--rate" {
            index += 1;
            let value = arguments
                .get(index)
                .ok_or_else(|| invalid_input("--rate requires a value"))?;
            if options.rate.replace(parse_rate(value)?).is_some() {
                return Err(invalid_input("--rate may only be provided once"));
            }
        } else if arguments[index] == "--help" || arguments[index] == "-h" {
            return Ok(Command::Help);
        } else if arguments[index].to_string_lossy().starts_with('-') {
            return Err(invalid_input(&format!(
                "unknown option: {}",
                arguments[index].to_string_lossy()
            )));
        } else {
            inputs.push(PathBuf::from(&arguments[index]));
        }
        index += 1;
    }
    if inputs.is_empty() {
        return Err(invalid_input(
            "at least one input file or directory is required",
        ));
    }
    let output = output.ok_or_else(|| invalid_input("--out is required"))?;
    if labels {
        Ok(Command::ExportLabelTemplate {
            inputs,
            output,
            options,
        })
    } else {
        Ok(Command::ExportFeatures {
            inputs,
            output,
            options,
        })
    }
}

fn parse_analyze_args(arguments: &[OsString]) -> Result<Command, Box<dyn Error>> {
    let mut path = None;
    let mut mode = OutputMode::Compact;
    let mut options = AnalysisOptions::default();
    let mut index = 0;
    while index < arguments.len() {
        let argument = &arguments[index];
        if argument == "--details" {
            if mode == OutputMode::Json {
                return Err(invalid_input("--details and --json cannot be combined"));
            }
            mode = OutputMode::Details;
        } else if argument == "--json" {
            if mode == OutputMode::Details {
                return Err(invalid_input("--details and --json cannot be combined"));
            }
            mode = OutputMode::Json;
        } else if argument == "--mods" {
            index += 1;
            let value = arguments
                .get(index)
                .ok_or_else(|| invalid_input("--mods requires a value"))?;
            parse_mods(value, &mut options.mods)?;
        } else if argument == "--rate" {
            index += 1;
            let value = arguments
                .get(index)
                .ok_or_else(|| invalid_input("--rate requires a value"))?;
            if options.rate.replace(parse_rate(value)?).is_some() {
                return Err(invalid_input("--rate may only be provided once"));
            }
        } else if argument.to_string_lossy().starts_with('-') {
            return Err(invalid_input(&format!(
                "unknown option: {}",
                argument.to_string_lossy()
            )));
        } else if path.replace(PathBuf::from(argument)).is_some() {
            return Err(invalid_input("only one beatmap path may be provided"));
        }
        index += 1;
    }
    path.map(|path| Command::Analyze {
        path,
        mode,
        options,
    })
    .ok_or_else(|| invalid_input("run with --help for usage"))
}

fn parse_mods(value: &OsStr, mods: &mut Vec<GameMod>) -> Result<(), Box<dyn Error>> {
    let value = value
        .to_str()
        .ok_or_else(|| invalid_input("--mods must be valid UTF-8"))?;
    if value.trim().is_empty() {
        return Err(invalid_input("--mods cannot be empty"));
    }
    for acronym in value.split(',') {
        mods.push(
            acronym
                .parse()
                .map_err(|message: String| invalid_input(&message))?,
        );
    }
    Ok(())
}

fn parse_rate(value: &OsStr) -> Result<f64, Box<dyn Error>> {
    let value = value
        .to_str()
        .ok_or_else(|| invalid_input("--rate must be valid UTF-8"))?;
    value
        .parse()
        .map_err(|_| invalid_input(&format!("invalid playback rate `{value}`")))
}

fn print_help() {
    println!("Analyze osu! beatmaps and export ML-ready datasets.");
    println!();
    println!("usage:");
    println!("  osu-map-analyzer [--details | --json] [--mods MODS] [--rate RATE] <map.osu>");
    println!("  osu-map-analyzer export-features <inputs>... [--mods MODS] [--rate RATE] --out <dataset.jsonl|dataset.csv>");
    println!(
        "  osu-map-analyzer export-label-template <inputs>... [--mods MODS] [--rate RATE] --out <labels.jsonl>"
    );
    println!(
        "  osu-map-analyzer merge-dataset --features <features.jsonl> --labels <labels.jsonl> --out <training.jsonl>"
    );
}

#[cfg(feature = "serialize")]
fn print_json(analysis: &BeatmapAnalysis) -> Result<(), Box<dyn Error>> {
    println!("{}", serde_json::to_string_pretty(analysis)?);
    Ok(())
}

#[cfg(not(feature = "serialize"))]
fn print_json(_: &BeatmapAnalysis) -> Result<(), Box<dyn Error>> {
    Err(invalid_input("--json requires the serialize feature"))
}

fn invalid_input(message: &str) -> Box<dyn Error> {
    io::Error::new(io::ErrorKind::InvalidInput, message).into()
}
