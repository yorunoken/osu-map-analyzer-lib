use crate::utils::{bpm, calculate_distance};
use rosu_map::{section::hit_objects::HitObject, Beatmap};
use std::collections::VecDeque;

pub struct Jump {
    map: Beatmap,
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct JumpAnalysis {
    pub overall_confidence: f64,
    pub total_jump_count: usize,
    pub max_jump_length: usize,
    pub short_jumps_count: usize,
    pub medium_jumps_count: usize,
    pub long_jumps_count: usize,
    pub peak_jump_density: f64,
    pub bpm_consistency: f64,
}

impl Jump {
    /// Creates a new jump analyzer for the given beatmap.
    ///
    /// # Arguments
    ///
    /// * `map` - The beatmap to analyze, type: rosu_map::Beatmap.
    pub fn new(map: Beatmap) -> Self {
        Self { map }
    }

    /// Analyzes the beatmap for jumps and returns a `JumpAnalysis`.
    ///
    /// # Example
    ///
    /// ```rs
    /// let path = Path::new("example-maps/jump-caffeinefighter.osu");
    /// let map = rosu_map::from_path::<rosu_map::Beatmap>(path).unwrap();

    /// let mut jump_analyzer = Jump::new(map);
    /// let analasis = jump_analyzer.analyze();
    /// println!("{:#?}", analasis);
    /// ```
    pub fn analyze(&mut self) -> JumpAnalysis {
        let bpm = bpm(
            self.map.hit_objects.last_mut(),
            &self.map.control_points.timing_points,
        );
        let beat_length = 60.0 / bpm * 1000.0;
        let expected_jump_interval = beat_length / 2.0; // 1/2ths

        let window_size = 200;
        let step_size = 50;
        let hit_objects = &self.map.hit_objects;

        let mut max_jump_length = 0;
        let mut peak_jump_density = 0.0_f64;

        let mut total_short_jump_count = 0;
        let mut total_medium_jump_count = 0;
        let mut total_long_jump_count = 0;

        let mut overall_bpm_consistency: f64 = 0.0;
        let mut total_jump_notes_length = 0;
        let mut total_jumps = 0;

        for window_start in (0..hit_objects.len()).step_by(step_size) {
            let window_end = (window_start + window_size).min(hit_objects.len());
            let window = &hit_objects[window_start..window_end];

            let (jumps_lengths, bpm_variations) =
                self.analyze_window(window, expected_jump_interval);

            let short_jumps = jumps_lengths
                .iter()
                .filter(|&&len| len > 2 && len < 10)
                .count();
            let medium_jumps = jumps_lengths
                .iter()
                .filter(|&&len| len >= 10 && len < 20)
                .count();
            let long_jumps = jumps_lengths.iter().filter(|&&len| len >= 20).count();

            total_short_jump_count += short_jumps;
            total_medium_jump_count += medium_jumps;
            total_long_jump_count += long_jumps;

            let total_jump_notes: usize = jumps_lengths.iter().sum();
            let jump_density = total_jump_notes as f64 / window.len() as f64;
            peak_jump_density = peak_jump_density.max(jump_density);

            max_jump_length = max_jump_length.max(*jumps_lengths.iter().max().unwrap_or(&0));

            total_jump_notes_length += jumps_lengths.iter().sum::<usize>();
            total_jumps += jumps_lengths.len();

            let bpm_consistency = if !bpm_variations.is_empty() {
                1.0 - (bpm_variations.iter().sum::<f64>() / bpm_variations.len() as f64)
                    / expected_jump_interval
            } else {
                0.0
            };

            overall_bpm_consistency = overall_bpm_consistency.max(bpm_consistency);
        }

        let average_jump_length = if total_jumps > 0 {
            total_jump_notes_length as f64 / total_jumps as f64
        } else {
            0.0
        };

        let jump_variety = (total_medium_jump_count * 2 + total_long_jump_count * 3) as f64
            / (total_short_jump_count + total_medium_jump_count + total_long_jump_count).max(1)
                as f64;

        let long_jump_ratio = total_long_jump_count as f64 / total_jumps as f64;

        let overall_confidence = (peak_jump_density * 0.4
            + overall_bpm_consistency * 0.2
            + jump_variety * 0.35
            + long_jump_ratio * 0.45
            + (average_jump_length / 3.0).min(1.0) * 0.3)
            .min(1.0);

        JumpAnalysis {
            long_jumps_count: total_long_jump_count,
            medium_jumps_count: total_medium_jump_count,
            short_jumps_count: total_short_jump_count,
            max_jump_length,
            total_jump_count: total_jumps,
            overall_confidence,
            peak_jump_density,
            bpm_consistency: overall_bpm_consistency,
        }
    }

    fn analyze_window(
        &self,
        window: &[HitObject],
        expected_interval: f64,
    ) -> (Vec<usize>, Vec<f64>) {
        let mut jumps_lengths = Vec::new();
        let mut curr_jump = VecDeque::new();
        let mut bpm_variations = Vec::new();
        let tolerance = 0.10; // 10% tolerance
        let distance_threshold = 120.0_f32;

        for pair in window.windows(2) {
            let obj1 = &pair[0];
            let obj2 = &pair[1];

            let time_diff = obj2.start_time - obj1.start_time;
            let distance = calculate_distance(obj1, obj2);

            // Check if the pair is between expected interval.
            if (time_diff - expected_interval).abs() / expected_interval <= tolerance
                && distance >= distance_threshold
            {
                curr_jump.push_back(time_diff);
                if curr_jump.len() > 1 {
                    let prev_diff = curr_jump[curr_jump.len() - 2];
                    bpm_variations.push((time_diff - prev_diff).abs());
                }
            } else if !curr_jump.is_empty() {
                jumps_lengths.push(curr_jump.len());
                curr_jump.clear();
            }
        }

        if !curr_jump.is_empty() {
            jumps_lengths.push(curr_jump.len());
        }

        (jumps_lengths, bpm_variations)
    }
}

#[cfg(test)]
mod jump_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_jump_analysis() {
        let path = Path::new("example-maps/jump-flowerdance.osu");
        let map = rosu_map::from_path::<rosu_map::Beatmap>(path).unwrap();

        let mut jump_analyzer = Jump::new(map);
        let analasis = jump_analyzer.analyze();
        println!("{:#?}", analasis);
    }
}
