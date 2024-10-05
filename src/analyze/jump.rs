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
    pub long_jumps: usize,
    pub medium_jumps: usize,
    pub short_jumps: usize,

    pub jump_density: f64,
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
    /// let analysis = jump_analyzer.analyze();
    /// println!("{:#?}", analasis);
    /// ```
    pub fn analyze(&mut self) -> JumpAnalysis {
        let bpm = bpm(
            self.map.hit_objects.last_mut(),
            &self.map.control_points.timing_points,
        );
        let beat_length = 60.0 / bpm * 1000.0;
        let expected_jump_interval = beat_length / 2.0; // 1/2ths
        let hit_objects = &self.map.hit_objects;

        let (consecutive_notes, bpm_variations) =
            self.calculate_consecutive_notes(hit_objects, expected_jump_interval);

        // Calculate jumps' lengths
        let short_jumps_amount = consecutive_notes
            .iter()
            .filter(|&&len| len >= 4 && len < 7)
            .count();
        let medium_jumps_amount = consecutive_notes
            .iter()
            .filter(|&&len| len >= 7 && len < 12)
            .count();
        let long_jumps_amount = consecutive_notes.iter().filter(|&&len| len >= 12).count();

        // Filter `consecutive_notes` to only have note amount higher than `5`,
        // because `consecutive_notes` returns the consecutive notes, `not` jumps.
        // Extremely short jumps could still be counted as jumps if I don't do this.
        let jumps_lengths: Vec<usize> = consecutive_notes
            .iter()
            .filter(|&&len| len >= 4)
            .map(|&len| len)
            .collect();

        let total_jump_notes: usize = jumps_lengths.iter().sum();
        let jump_density = total_jump_notes as f64 / hit_objects.len() as f64;

        let max_jump_length = *consecutive_notes.iter().max().unwrap_or(&0);
        let total_jumps_amount = short_jumps_amount + medium_jumps_amount + long_jumps_amount;

        let bpm_consistency = if !bpm_variations.is_empty() {
            1.0 - (bpm_variations.iter().sum::<f64>() / bpm_variations.len() as f64)
                / expected_jump_interval
        } else {
            0.0
        };

        let average_jump_length = if total_jumps_amount > 0 {
            total_jump_notes as f64 / total_jumps_amount as f64
        } else {
            0.0
        };

        let jump_variety = (medium_jumps_amount * 2 + long_jumps_amount * 3) as f64
            / (short_jumps_amount + medium_jumps_amount + long_jumps_amount).max(1) as f64;

        let long_jump_ratio = long_jumps_amount as f64 / total_jumps_amount as f64;

        let overall_confidence = (jump_density * 0.4
            + bpm_consistency * 0.2
            + jump_variety * 0.35
            + long_jump_ratio * 0.45
            + (average_jump_length / 3.0).min(1.0) * 0.3)
            .min(1.0);

        JumpAnalysis {
            long_jumps: long_jumps_amount,
            medium_jumps: medium_jumps_amount,
            short_jumps: short_jumps_amount,
            max_jump_length,
            total_jump_count: total_jumps_amount,
            overall_confidence,
            jump_density,
            bpm_consistency,
        }
    }

    fn calculate_consecutive_notes(
        &self,
        hit_objects: &[HitObject],
        expected_interval: f64,
    ) -> (Vec<usize>, Vec<f64>) {
        let mut jumps_lengths = Vec::new();
        let mut curr_jump = VecDeque::new();
        let mut bpm_variations = Vec::new();
        let tolerance = 0.10; // 10% tolerance
        let distance_threshold = 120.0_f32;

        for pair in hit_objects.windows(2) {
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
