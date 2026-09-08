use rosu_map::{
    section::{
        hit_objects::{CurveBuffers, HitObjectKind},
        timing_points::TimingPoint,
    },
    util::Pos,
    Beatmap,
};

use crate::AnalysisConfig;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ObjectKind {
    Circle,
    Slider,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ObjectFeature {
    pub(crate) start_time: f64,
    pub(crate) position: Pos,
    pub(crate) kind: ObjectKind,
    pub(crate) slider_duration: f64,
    pub(crate) slider_repeats: usize,
    pub(crate) slider_travel: f64,
    pub(crate) slider_velocity: f64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Transition {
    pub(crate) from: usize,
    pub(crate) to: usize,
    pub(crate) delta_ms: f64,
    pub(crate) elapsed_beats: f64,
    pub(crate) normalized_distance: f64,
    pub(crate) angle_degrees: Option<f64>,
    pub(crate) section: usize,
}

#[derive(Debug)]
pub(crate) struct FeatureSet {
    pub(crate) objects: Vec<ObjectFeature>,
    pub(crate) transitions: Vec<Transition>,
    pub(crate) circle_count: usize,
    pub(crate) duration_ms: f64,
    pub(crate) start_time: f64,
    pub(crate) section_object_counts: Vec<usize>,
}

impl FeatureSet {
    pub(crate) fn extract(map: &Beatmap, config: &AnalysisConfig) -> Self {
        let mut source_objects: Vec<_> = map.hit_objects.iter().collect();
        source_objects.sort_by(|left, right| left.start_time.total_cmp(&right.start_time));

        let circle_radius = (54.4 - 4.48 * f64::from(map.circle_size)).max(1.0);
        let start_time = source_objects
            .iter()
            .find_map(|object| playable_parts(object).map(|_| object.start_time))
            .filter(|time| time.is_finite())
            .unwrap_or(0.0);
        let mut curve_buffers = CurveBuffers::default();
        let mut objects = Vec::new();
        let mut transitions: Vec<Transition> = Vec::new();
        let mut previous = None;
        let mut circle_count = 0;
        let mut section_object_counts = Vec::new();

        for object in source_objects {
            if !object.start_time.is_finite() {
                previous = None;
                continue;
            }

            let Some((position, kind)) = playable_parts(object) else {
                previous = None;
                continue;
            };

            if !position.x.is_finite() || !position.y.is_finite() {
                previous = None;
                continue;
            }

            let (slider_duration, slider_repeats, slider_travel, slider_velocity) =
                match &object.kind {
                    HitObjectKind::Slider(slider) => {
                        let path_distance = slider.path.borrowed_curve(&mut curve_buffers).dist();
                        let span_count = f64::from(slider.span_count().max(1));
                        let travel = path_distance * span_count;
                        let duration = if slider.velocity.is_finite() && slider.velocity > 0.0 {
                            travel / slider.velocity
                        } else {
                            0.0
                        };
                        let velocity = map
                            .control_points
                            .difficulty_point_at(object.start_time)
                            .map_or(1.0, |point| point.slider_velocity);

                        (
                            duration.max(0.0),
                            slider.repeat_count.max(0) as usize,
                            travel.max(0.0),
                            velocity,
                        )
                    }
                    _ => (0.0, 0, 0.0, 1.0),
                };

            if kind == ObjectKind::Circle {
                circle_count += 1;
            }

            let to = objects.len();
            let section = section_index(object.start_time, start_time, config.peak_window_ms);

            if section_object_counts.len() <= section {
                section_object_counts.resize(section + 1, 0);
            }
            section_object_counts[section] += 1;

            objects.push(ObjectFeature {
                start_time: object.start_time,
                position,
                kind,
                slider_duration,
                slider_repeats,
                slider_travel,
                slider_velocity,
            });

            if let Some(from) = previous {
                let previous_object: &ObjectFeature = &objects[from];
                let delta_ms = object.start_time - previous_object.start_time;

                if delta_ms.is_finite() && delta_ms > 0.0 {
                    let movement = position - previous_object.position;
                    let normalized_distance = f64::from(movement.length()) / circle_radius;
                    let angle_degrees = transitions.last().and_then(|previous_transition| {
                        (previous_transition.to == from).then(|| {
                            let earlier = objects[previous_transition.from].position;
                            movement_angle(previous_object.position - earlier, movement)
                        })
                    }).flatten();

                    transitions.push(Transition {
                        from,
                        to,
                        delta_ms,
                        elapsed_beats: elapsed_beats(
                            &map.control_points.timing_points,
                            previous_object.start_time,
                            object.start_time,
                        ),
                        normalized_distance,
                        angle_degrees,
                        section: section_index(
                            previous_object.start_time,
                            start_time,
                            config.peak_window_ms,
                        ),
                    });
                }
            }

            previous = Some(to);
        }

        let duration_ms = objects
            .last()
            .map_or(0.0, |object| (object.start_time - start_time).max(0.0));

        Self {
            objects,
            transitions,
            circle_count,
            duration_ms,
            start_time,
            section_object_counts,
        }
    }
}

fn playable_parts(object: &rosu_map::section::hit_objects::HitObject) -> Option<(Pos, ObjectKind)> {
    match &object.kind {
        HitObjectKind::Circle(circle) => Some((circle.pos, ObjectKind::Circle)),
        HitObjectKind::Slider(slider) => Some((slider.pos, ObjectKind::Slider)),
        HitObjectKind::Spinner(_) | HitObjectKind::Hold(_) => None,
    }
}

fn section_index(time: f64, start_time: f64, window_ms: f64) -> usize {
    ((time - start_time).max(0.0) / window_ms).floor() as usize
}

fn movement_angle(previous: Pos, current: Pos) -> Option<f64> {
    let denominator = f64::from(previous.length()) * f64::from(current.length());

    if denominator <= f64::EPSILON {
        return None;
    }

    let cosine = (f64::from(previous.dot(current)) / denominator).clamp(-1.0, 1.0);
    Some(cosine.acos().to_degrees())
}

fn elapsed_beats(points: &[TimingPoint], start: f64, end: f64) -> f64 {
    let mut cursor = start;
    let mut beat_len = active_beat_len(points, start);
    let mut beats = 0.0;

    for point in points
        .iter()
        .filter(|point| point.time > start && point.time < end)
    {
        beats += (point.time - cursor) / beat_len;
        cursor = point.time;
        beat_len = point.beat_len;
    }

    beats + (end - cursor) / beat_len
}

fn active_beat_len(points: &[TimingPoint], time: f64) -> f64 {
    points
        .iter()
        .rev()
        .find(|point| point.time <= time)
        .or_else(|| points.first())
        .map_or(TimingPoint::DEFAULT_BEAT_LEN, |point| point.beat_len)
}

#[cfg(test)]
mod tests {
    use rosu_map::Beatmap;

    use super::FeatureSet;
    use crate::AnalysisConfig;

    fn beatmap(timing_points: &[&str], hit_objects: &[&str]) -> Beatmap {
        let source = format!(
            "osu file format v14\n\n\
             [General]\nMode:0\n\n\
             [Difficulty]\nCircleSize:4\nSliderMultiplier:1.4\nSliderTickRate:1\n\n\
             [TimingPoints]\n{}\n\
             [HitObjects]\n{}\n",
            timing_points.join("\n"),
            hit_objects.join("\n"),
        );

        Beatmap::from_bytes(source.as_bytes()).unwrap()
    }

    #[test]
    fn integrates_beats_across_a_timing_change() {
        let map = beatmap(
            &["0,500,4,2,1,50,1,0", "1000,250,4,2,1,50,1,0"],
            &["64,192,750,1,0", "128,192,1125,1,0"],
        );

        let features = FeatureSet::extract(&map, &AnalysisConfig::default());

        assert!((features.transitions[0].elapsed_beats - 1.0).abs() < 1e-9);
    }

    #[test]
    fn normalizes_distance_by_circle_radius() {
        let map = beatmap(
            &["0,500,4,2,1,50,1,0"],
            &["0,192,0,1,0", "100,192,250,1,0"],
        );

        let features = FeatureSet::extract(&map, &AnalysisConfig::default());

        assert!((features.transitions[0].normalized_distance - 2.741_228).abs() < 1e-5);
    }

    #[test]
    fn spinner_breaks_a_transition_sequence() {
        let map = beatmap(
            &["0,500,4,2,1,50,1,0"],
            &[
                "64,192,0,1,0",
                "256,192,125,8,0,375",
                "128,192,500,1,0",
            ],
        );

        assert!(
            FeatureSet::extract(&map, &AnalysisConfig::default())
                .transitions
                .is_empty()
        );
    }

    #[test]
    fn sorts_objects_without_mutating_the_map() {
        let mut map = beatmap(
            &["0,500,4,2,1,50,1,0"],
            &["64,192,0,1,0", "128,192,250,1,0"],
        );
        map.hit_objects.swap(0, 1);
        let original_first = map.hit_objects[0].start_time;

        let features = FeatureSet::extract(&map, &AnalysisConfig::default());

        assert_eq!(features.objects[0].start_time, 0.0);
        assert_eq!(map.hit_objects[0].start_time, original_first);
    }
}
