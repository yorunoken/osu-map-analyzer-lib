use crate::utils::bpm;
use rosu_map::{section::hit_objects::HitObject, Beatmap};
use std::collections::VecDeque;

pub struct Stream {
    map: Beatmap,
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct StreamAnalysis {
    pub overall_confidence: f64,
    pub short_streams: usize,
    pub medium_streams: usize,
    pub long_streams: usize,
    pub max_stream_length: usize,
    pub peak_stream_density: f64,
    pub bpm_consistency: f64,
}

impl Stream {
    /// Creates a new stream analyzer for the given beatmap.
    ///
    /// # Arguments
    ///
    /// * `map` - The beatmap to analyze, type: rosu_map::Beatmap.
    pub fn new(map: Beatmap) -> Self {
        Self { map }
    }

    /// Analyzes the beatmap for streams and returns a `StreamAnalysis`.
    ///
    /// # Example
    ///
    /// ```rs
    /// let path = Path::new("example-maps/jump-caffeinefighter.osu");
    /// let map = rosu_map::from_path::<rosu_map::Beatmap>(path).unwrap();
    ///
    /// let mut stream_analyzer = Stream::new(map);
    /// let analasis = stream_analyzer.analyze();
    /// println!("{:#?}", analasis);
    /// ```
    pub fn analyze(&mut self) -> StreamAnalysis {
        let bpm = bpm(None, &self.map.control_points.timing_points);
        let beat_length = 60.0 / bpm * 1000.0;
        let expected_stream_interval = beat_length / 4.0; // 1/4ths

        let window_size = 200;
        let step_size = 50;
        let hit_objects = &self.map.hit_objects;

        let mut max_stream_length = 0;
        let mut total_short_stream_count = 0;
        let mut total_medium_stream_count = 0;
        let mut total_long_stream_count = 0;
        let mut peak_stream_density: f64 = 0.0;
        let mut overall_bpm_consistency: f64 = 0.0;
        let mut total_stream_length = 0;
        let mut total_streams = 0;

        for window_start in (0..hit_objects.len()).step_by(step_size) {
            let window_end = (window_start + window_size).min(hit_objects.len());
            let window = &hit_objects[window_start..window_end];

            let (stream_lengths, bpm_variations) =
                self.analyze_window(window, expected_stream_interval);

            let short_streams = stream_lengths
                .iter()
                .filter(|&&len| len >= 5 && len < 10)
                .count();
            let medium_streams = stream_lengths
                .iter()
                .filter(|&&len| len >= 10 && len < 20)
                .count();
            let long_streams = stream_lengths.iter().filter(|&&len| len >= 20).count();

            max_stream_length = max_stream_length.max(*stream_lengths.iter().max().unwrap_or(&0));
            total_short_stream_count += short_streams;
            total_medium_stream_count += medium_streams;
            total_long_stream_count += long_streams;

            total_stream_length += stream_lengths.iter().sum::<usize>();
            total_streams += stream_lengths.len();

            let total_stream_notes: usize = stream_lengths.iter().sum();
            let stream_density = total_stream_notes as f64 / window.len() as f64;
            peak_stream_density = peak_stream_density.max(stream_density);

            let bpm_consistency = if !bpm_variations.is_empty() {
                1.0 - (bpm_variations.iter().sum::<f64>() / bpm_variations.len() as f64)
                    / expected_stream_interval
            } else {
                0.0
            };
            overall_bpm_consistency = overall_bpm_consistency.max(bpm_consistency);
        }

        let average_stream_length = if total_streams > 0 {
            total_stream_length as f64 / total_streams as f64
        } else {
            0.0
        };

        let stream_variety = (total_medium_stream_count * 2 + total_long_stream_count * 3) as f64
            / (total_short_stream_count + total_medium_stream_count + total_long_stream_count)
                .max(1) as f64;

        let long_stream_ratio = total_long_stream_count as f64 / total_streams.max(1) as f64;

        let overall_confidence = (peak_stream_density * 0.3
            + overall_bpm_consistency * 0.2
            + stream_variety * 0.2
            + long_stream_ratio * 0.2
            + (average_stream_length / 5.0).min(1.0) * 0.2)
            .min(1.0);

        StreamAnalysis {
            overall_confidence,
            short_streams: total_short_stream_count,
            medium_streams: total_medium_stream_count,
            long_streams: total_long_stream_count,
            max_stream_length,
            peak_stream_density,
            bpm_consistency: overall_bpm_consistency,
        }
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

#[cfg(test)]
mod stream_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_stream_analysis() {
        let path = Path::new("example-maps/alt-sasageyo.osu");
        let map = rosu_map::from_path::<rosu_map::Beatmap>(path).unwrap();

        let mut stream_analyzer = Stream::new(map);
        let analasis = stream_analyzer.analyze();
        println!("{:#?}", analasis);
    }
}
