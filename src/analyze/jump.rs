use std::collections::VecDeque;

use rosu_map::{section::hit_objects::HitObject, Beatmap};

pub struct Jump {
    map: Beatmap,
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct JumpAnalysis {
    pub overall_confidence: f64,
    pub total_jump_count: usize,
    pub peak_jump_length: usize,
}

impl Jump {
    /// Creates a new jump analyzer for the given beatmap.
    ///
    /// # Arguments
    ///
    /// * `map` - The beatmap to analyze, type: rosu_map::Beatmap.
    fn new(map: Beatmap) -> Self {
        Self { map }
    }

    fn analyze_window(
        &self,
        window: &[HitObject],
        expected_interval: f64,
    ) -> (Vec<usize>, Vec<f64>) {
        let mut stream_lengths = Vec::new();
        let mut current_stream = VecDeque::new();
        let mut bpm_variations = Vec::new();
        let tolerance = 0.10; // 10% tolerance

        for pair in window.windows(2) {
            let time_diff = pair[1].start_time - pair[0].start_time;

            // Check if the pair is between expected interval.
            if (time_diff - expected_interval).abs() / expected_interval <= tolerance {
                current_stream.push_back(time_diff);
                if current_stream.len() > 1 {
                    let prev_diff = current_stream[current_stream.len() - 2];
                    bpm_variations.push((time_diff - prev_diff).abs());
                }
            } else if !current_stream.is_empty() {
                stream_lengths.push(current_stream.len());
                current_stream.clear();
            }
        }

        if !current_stream.is_empty() {
            stream_lengths.push(current_stream.len());
        }

        (stream_lengths, bpm_variations)
    }
}
