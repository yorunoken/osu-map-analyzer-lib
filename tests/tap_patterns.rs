mod support;

use osu_map_analyzer::Analyzer;

#[test]
fn counts_notes_instead_of_edges() {
    let map = support::circle_run(7, 125.0, 40.0);
    let result = Analyzer::new(&map).analyze().unwrap();

    assert_eq!(result.stream.longest, 7);
    assert_eq!(result.stream.notes, 7);
    assert_eq!(result.stream.segments, 1);
}

#[test]
fn reports_a_240_note_stream_without_capping_it() {
    let map = support::circle_run(240, 100.0, 24.0);

    assert_eq!(Analyzer::new(&map).analyze().unwrap().stream.longest, 240);
}

#[test]
fn classifies_three_to_five_notes_as_bursts() {
    let map = support::separated_circle_runs(&[3, 4, 5], 125.0, 1_000.0);
    let result = Analyzer::new(&map).analyze().unwrap();

    assert_eq!(result.burst.segments, 3);
    assert_eq!(result.burst.notes, 12);
    assert_eq!(result.burst.longest, 5);
    assert_eq!(result.stream.segments, 0);
}

#[test]
fn preserves_a_stream_across_a_bpm_change() {
    let times = [750, 875, 1_000, 1_063, 1_125, 1_188, 1_250, 1_313];
    let objects = times
        .iter()
        .enumerate()
        .map(|(index, &time)| support::circle(128 + (index % 2) as i32 * 40, 192, time))
        .collect::<Vec<_>>();
    let map = support::beatmap(&["0,500,4,2,1,50,1,0", "1000,250,4,2,1,50,1,0"], &objects);

    assert_eq!(Analyzer::new(&map).analyze().unwrap().stream.longest, 8);
}

#[test]
fn a_slider_splits_fast_circle_runs() {
    let map = support::beatmap(
        &["0,500,4,2,1,50,1,0"],
        &[
            support::circle(128, 192, 0),
            support::circle(168, 192, 125),
            support::slider(128, 192, 250, 228, 192),
            support::circle(168, 192, 375),
            support::circle(128, 192, 500),
        ],
    );

    let result = Analyzer::new(&map).analyze().unwrap();
    assert_eq!(result.stream.segments, 0);
    assert_eq!(result.burst.segments, 0);
}
