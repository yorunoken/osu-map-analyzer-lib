mod support;

use osu_map_analyzer::rosu_map::section::general::GameMode;
use osu_map_analyzer::{AnalysisConfig, AnalysisError, Analyzer};

#[test]
fn rejects_non_standard_maps() {
    let mut map = support::beatmap(&["0,500,4,2,1,50,1,0"], &[support::circle(256, 192, 0)]);
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
    let map = support::beatmap(&["0,500,4,2,1,50,1,0"], &[support::circle(256, 192, 0)]);
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
    let map = support::beatmap(&["0,500,4,2,1,50,1,0"], &[support::circle(256, 192, 0)]);
    let config = AnalysisConfig {
        primary_score_min: 1.1,
        ..AnalysisConfig::default()
    };

    assert!(matches!(
        Analyzer::with_config(&map, config),
        Err(AnalysisError::InvalidConfig("primary_score_min"))
    ));
}

#[test]
fn rejects_each_remaining_invalid_config_field() {
    let map = support::beatmap(&["0,500,4,2,1,50,1,0"], &[support::circle(256, 192, 0)]);
    let cases = [
        (
            "fast_interval_beats",
            AnalysisConfig {
                fast_interval_beats: f64::NAN,
                ..AnalysisConfig::default()
            },
        ),
        (
            "rhythm_tolerance",
            AnalysisConfig {
                rhythm_tolerance: 1.1,
                ..AnalysisConfig::default()
            },
        ),
        (
            "stream_min_notes",
            AnalysisConfig {
                stream_min_notes: 3,
                ..AnalysisConfig::default()
            },
        ),
        (
            "jump_min_distance",
            AnalysisConfig {
                jump_min_distance: 0.0,
                ..AnalysisConfig::default()
            },
        ),
        (
            "jump_max_interval_ms",
            AnalysisConfig {
                jump_max_interval_ms: f64::INFINITY,
                ..AnalysisConfig::default()
            },
        ),
        (
            "jump_max_interval_beats",
            AnalysisConfig {
                jump_max_interval_beats: -1.0,
                ..AnalysisConfig::default()
            },
        ),
        (
            "peak_window_ms",
            AnalysisConfig {
                peak_window_ms: 0.0,
                ..AnalysisConfig::default()
            },
        ),
    ];

    for (field, config) in cases {
        assert_eq!(
            Analyzer::with_config(&map, config).err(),
            Some(AnalysisError::InvalidConfig(field))
        );
    }
}
