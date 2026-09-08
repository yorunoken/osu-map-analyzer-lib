mod support;

use osu_map_analyzer::Analyzer;

#[test]
fn spaced_deathstream_is_not_a_jump_sequence() {
    let map = support::circle_run(240, 100.0, 120.0);
    let result = Analyzer::new(&map).analyze().unwrap();

    assert_eq!(result.stream.longest, 240);
    assert_eq!(result.jump.segments, 0);
}

#[test]
fn detects_a_half_beat_jump_sequence() {
    let map = support::circle_run(12, 250.0, 180.0);
    let result = Analyzer::new(&map).analyze().unwrap();

    assert_eq!(result.jump.longest, 12);
    assert_eq!(result.jump.segments, 1);
    assert!(result.jump.average_distance > 4.0);
}

#[test]
fn the_same_pixel_distance_changes_with_circle_size() {
    let small_circles = support::jump_pair_with_cs(5.0, 100.0);
    let large_circles = support::jump_pair_with_cs(2.0, 100.0);

    assert!(
        Analyzer::new(&small_circles).analyze().unwrap().jump.score
            > Analyzer::new(&large_circles).analyze().unwrap().jump.score
    );
}

#[test]
fn slider_heads_can_participate_in_jumps() {
    let map = support::beatmap(
        &["0,500,4,2,1,50,1,0"],
        &[
            support::circle(64, 192, 0),
            support::slider(244, 192, 250, 344, 192),
            support::circle(64, 192, 500),
        ],
    );

    assert_eq!(Analyzer::new(&map).analyze().unwrap().jump.longest, 3);
}
