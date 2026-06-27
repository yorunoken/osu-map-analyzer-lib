use std::{env, error::Error, io, path::PathBuf};

use osu_map_analyzer::{AnalysisOptions, BeatmapAnalysis, BeatmapAnalyzer};

#[derive(Clone, Copy, PartialEq, Eq)]
enum OutputMode {
    Compact,
    Details,
    Json,
}

fn main() -> Result<(), Box<dyn Error>> {
    let (path, mode) = parse_args()?;
    let analysis = BeatmapAnalyzer::from_path(path)?.analyze(AnalysisOptions::default())?;

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

fn parse_args() -> Result<(PathBuf, OutputMode), Box<dyn Error>> {
    let mut path = None;
    let mut mode = OutputMode::Compact;

    for argument in env::args_os().skip(1) {
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
        } else if argument == "--help" || argument == "-h" {
            print_help();
            std::process::exit(0);
        } else if path.replace(PathBuf::from(argument)).is_some() {
            return Err(invalid_input("only one beatmap path may be provided"));
        }
    }

    path.map(|path| (path, mode))
        .ok_or_else(|| invalid_input("usage: osu-map-analyzer [--details | --json] <map.osu>"))
}

fn print_help() {
    println!("Analyze an osu! beatmap and report extracted features.");
    println!();
    println!("usage: osu-map-analyzer [--details | --json] <map.osu>");
    println!();
    println!("  --details  Print grouped metrics and peak sections");
    println!("  --json     Print the complete BeatmapAnalysis as JSON");
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
