mod support;

#[cfg(feature = "serde")]
use osu_map_analyzer::{Analyzer, MapAnalysis};

#[cfg(feature = "serde")]
#[test]
fn map_analysis_round_trips_through_json() {
    let map = support::circle_run(12, 125.0, 40.0);
    let analysis = Analyzer::new(&map).analyze().unwrap();
    let json = serde_json::to_string(&analysis).unwrap();

    assert_eq!(
        serde_json::from_str::<MapAnalysis>(&json).unwrap(),
        analysis
    );
}
