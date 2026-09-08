mod support;

use osu_map_analyzer::{Analyzer, Pattern};

#[test]
fn slider_metrics_include_repeats_travel_and_duration() {
    let map = support::slider_map();
    let result = Analyzer::new(&map).analyze().unwrap();

    assert_eq!(result.slider.count, 3);
    assert_eq!(result.slider.repeats, 2);
    assert!(result.slider.travel_distance > 0.0);
    assert!(result.slider.duration_ratio > 0.0);
}

#[test]
fn tech_requires_combined_rhythm_and_geometry_evidence() {
    let simple = support::circle_run(16, 250.0, 180.0);
    let technical = support::technical_map();

    assert!(
        Analyzer::new(&technical).analyze().unwrap().tech.score
            > Analyzer::new(&simple).analyze().unwrap().tech.score
    );
}

#[test]
fn rankings_are_complete_descending_and_stable() {
    let map = support::mixed_map();
    let result = Analyzer::new(&map).analyze().unwrap();

    assert_eq!(result.patterns.len(), 5);
    assert!(result
        .patterns
        .windows(2)
        .all(|pair| pair[0].score >= pair[1].score));
    assert_eq!(result.primary, Some(result.patterns[0].pattern));
    assert!(result
        .patterns
        .iter()
        .all(|entry| (0.0..=1.0).contains(&entry.score)));
}

#[test]
fn a_patternless_map_has_no_primary_pattern() {
    let map = support::beatmap(&["0,500,4,2,1,50,1,0"], &[support::circle(256, 192, 0)]);
    let result = Analyzer::new(&map).analyze().unwrap();

    assert_eq!(result.primary, None);
    assert_eq!(result.patterns[0].pattern, Pattern::Jump);
    assert!(result.patterns.iter().all(|entry| entry.score == 0.0));
}

#[test]
fn slider_peak_identifies_the_busy_window() {
    let map = support::beatmap(
        &["0,500,4,2,1,50,1,0"],
        &[
            support::circle(64, 192, 0),
            support::circle(128, 192, 2_100),
            support::slider(64, 192, 4_100, 164, 192),
            support::slider(256, 192, 4_400, 356, 192),
            support::slider(64, 192, 4_700, 164, 192),
        ],
    );
    let peak = Analyzer::new(&map).analyze().unwrap().slider.peak.unwrap();

    assert_eq!(peak.start_time_ms, 4_000.0);
    assert_eq!(peak.score, 1.0);
}
