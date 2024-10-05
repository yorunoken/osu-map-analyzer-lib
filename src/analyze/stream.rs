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
    pub stream_density: f64,
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
        let bpm = bpm(
            self.map.hit_objects.last_mut(),
            &self.map.control_points.timing_points,
        );
        let beat_length = 60.0 / bpm * 1000.0;
        let expected_stream_interval = beat_length / 4.0; // 1/4ths

        let hit_objects = &self.map.hit_objects;

        let (consecutive_notes, bpm_variations) =
            self.calculate_consecutive_notes(hit_objects, expected_stream_interval);

        let bursts_amount = consecutive_notes
            .iter()
            .filter(|&&len| len >= 3 && len <= 5)
            .count();

        let short_streams_amount = consecutive_notes
            .iter()
            .filter(|&&len| len >= 6 && len < 10)
            .count();
        let medium_streams_amount = consecutive_notes
            .iter()
            .filter(|&&len| len >= 10 && len < 20)
            .count();
        let long_streams_amount = consecutive_notes.iter().filter(|&&len| len >= 20).count();

        // Filter `consecutive_notes` to only have note amount higher than `6`,
        // because `consecutive_notes` returns the consecutive notes, `not` streams.
        // Bursts cont as consecutive, and so do doubles.
        let streams_lengths: Vec<usize> = consecutive_notes
            .iter()
            .filter(|&&len| len >= 6)
            .map(|&len| len)
            .collect();

        let total_stream_notes: usize = streams_lengths.iter().sum();
        let max_stream_length = *streams_lengths.iter().max().unwrap_or(&0);

        let total_streams_amount =
            short_streams_amount + medium_streams_amount + long_streams_amount;

        let stream_density = total_stream_notes as f64 / hit_objects.len() as f64;

        // How consistent streams' bpm variations are (this is usually between 1 and 0)
        let bpm_consistency = if !bpm_variations.is_empty() {
            1.0 - (bpm_variations.iter().sum::<f64>() / bpm_variations.len() as f64)
                / expected_stream_interval
        } else {
            0.0
        };

        let average_stream_length = if total_streams_amount > 0 {
            total_stream_notes as f64 / total_streams_amount as f64
        } else {
            0.0
        };

        let stream_variety = (medium_streams_amount * 2 + long_streams_amount * 3) as f64
            / (short_streams_amount + medium_streams_amount + long_streams_amount).max(1) as f64;

        let long_stream_ratio = long_streams_amount as f64 / total_streams_amount.max(1) as f64;

        let overall_confidence = (stream_density * 0.3
            + bpm_consistency * 0.2
            + stream_variety * 0.2
            + long_stream_ratio * 0.2
            + (average_stream_length / 5.0).min(1.0) * 0.2)
            .min(1.0);

        StreamAnalysis {
            overall_confidence,
            short_streams: short_streams_amount,
            medium_streams: medium_streams_amount,
            long_streams: long_streams_amount,
            max_stream_length,
            stream_density,
            bpm_consistency,
        }
    }

    fn calculate_consecutive_notes(
        &self,
        hit_objects: &[HitObject],
        expected_interval: f64,
    ) -> (Vec<usize>, Vec<f64>) {
        let mut stream_lengths = Vec::new();
        let mut current_stream = VecDeque::new();
        let mut bpm_variations = Vec::new();
        let tolerance = 0.10; // 10% tolerance

        // Look at streams in pairs
        // We do this so we can see if the note next to the curr note is a stream
        //
        // let notes = [1, 2, 3, 4, 5];
        // [1, 2] => [2, 3] => [3, 4] => [4, 5]
        // and then we look at their time differences, and if they're within our intervals, it counts as a consecutive note.
        for pair in hit_objects.windows(2) {
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
