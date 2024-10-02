use std::collections::HashMap;

use rosu_map::section::{hit_objects::HitObject, timing_points::TimingPoint};

pub fn bpm(last_hit_object: Option<&mut HitObject>, timing_points: &[TimingPoint]) -> f64 {
    let last_time = last_hit_object
        .map(HitObject::end_time)
        .or_else(|| timing_points.last().map(|t| t.time))
        .unwrap_or(0.0);

    let mut bpm_points = BeatLenDuration::new(last_time);

    match timing_points {
        [curr] => bpm_points.add(curr.beat_len, 0.0, last_time),
        [curr, next, ..] => bpm_points.add(curr.beat_len, 0.0, next.time),
        [] => {}
    }

    timing_points
        .iter()
        .skip(1)
        .zip(timing_points.iter().skip(2).map(|t| t.time))
        .for_each(|(curr, next_time)| bpm_points.add(curr.beat_len, curr.time, next_time));

    if let [.., _, curr] = timing_points {
        bpm_points.add(curr.beat_len, curr.time, last_time);
    }

    let most_common_beat_len = bpm_points
        .map
        .into_iter()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map_or(0.0, |(beatmap_len, _)| f64::from_bits(beatmap_len));

    (60_000.0 / most_common_beat_len).max(1.0)
}

struct BeatLenDuration {
    last_time: f64,
    map: HashMap<u64, f64>,
}

impl BeatLenDuration {
    fn new(last_time: f64) -> Self {
        Self {
            last_time,
            map: HashMap::default(),
        }
    }

    fn add(&mut self, beat_len: f64, curr_time: f64, next_time: f64) {
        let beat_len = (1000.0 * beat_len).round() / 1000.0;
        let entry = self.map.entry(beat_len.to_bits()).or_default();

        if curr_time <= self.last_time {
            *entry += next_time - curr_time;
        }
    }
}
