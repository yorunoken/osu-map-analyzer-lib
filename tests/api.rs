mod support;

use osu_map_analyzer::{AnalysisConfig, AnalysisError, Analyzer};
use osu_map_analyzer::rosu_map::section::general::GameMode;

#[test]
fn rejects_non_standard_maps() {
    let mut map = support::beatmap(
        &["0,500,4,2,1,50,1,0"],
        &[support::circle(256, 192, 0)],
    );
    map.mode = GameMode::Mania;

    assert!(matches!(
        Analyzer::new(&map).analyze(),
        Err(AnalysisError::UnsupportedMode(GameMode::Mania))
    ));
}

#[test]
fn rejects_an_empty_map() {
    let map = support::beatmap(&["0,500,4,2,1,50,1,0"], &[]);

    assert_eq!(Analyzer::new(&map).analyze(), Err(AnalysisError::EmptyMap));
}

#[test]
fn rejects_non_positive_fast_interval() {
    let map = support::beatmap(
        &["0,500,4,2,1,50,1,0"],
        &[support::circle(256, 192, 0)],
    );
    let config = AnalysisConfig {
        fast_interval_ms: 0.0,
        ..AnalysisConfig::default()
    };

    assert!(matches!(
        Analyzer::with_config(&map, config),
        Err(AnalysisError::InvalidConfig("fast_interval_ms"))
    ));
}

#[test]
fn rejects_an_invalid_primary_score_floor() {
    let map = support::beatmap(
        &["0,500,4,2,1,50,1,0"],
        &[support::circle(256, 192, 0)],
    );
    let config = AnalysisConfig {
        primary_score_min: 1.1,
        ..AnalysisConfig::default()
    };

    assert!(matches!(
        Analyzer::with_config(&map, config),
        Err(AnalysisError::InvalidConfig("primary_score_min"))
    ));
}
