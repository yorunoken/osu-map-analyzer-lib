use rosu_map::{section::general::GameMode, Beatmap};

use crate::{
    features::FeatureSet,
    patterns::{jump, slider, tap, tech},
    AnalysisConfig, AnalysisError, MapAnalysis, Pattern, PatternScore,
};

/// Analyzer for one borrowed osu!standard beatmap.
pub struct Analyzer<'map> {
    map: &'map Beatmap,
    config: AnalysisConfig,
}

impl<'map> Analyzer<'map> {
    /// Create an analyzer with [`AnalysisConfig::default`].
    pub fn new(map: &'map Beatmap) -> Self {
        Self {
            map,
            config: AnalysisConfig::default(),
        }
    }

    /// Create an analyzer with validated custom thresholds.
    pub fn with_config(
        map: &'map Beatmap,
        config: AnalysisConfig,
    ) -> Result<Self, AnalysisError> {
        Ok(Self {
            map,
            config: config.validate()?,
        })
    }

    /// Analyze the beatmap.
    pub fn analyze(&self) -> Result<MapAnalysis, AnalysisError> {
        if self.map.mode != GameMode::Osu {
            return Err(AnalysisError::UnsupportedMode(self.map.mode));
        }

        self.config.validate()?;

        let features = FeatureSet::extract(self.map, &self.config);

        if features.objects.is_empty() {
            return Err(AnalysisError::EmptyMap);
        }

        let tap = tap::detect(&features, &self.config);
        let jump = jump::detect(&features, &tap.stream_edges, &self.config);
        let slider = slider::detect(&features, &self.config);
        let tech = tech::detect(&features, &self.config);
        let mut patterns = vec![
            PatternScore {
                pattern: Pattern::Jump,
                score: jump.score,
                objects: jump.notes,
                segments: jump.segments,
                peak: jump.peak,
            },
            PatternScore {
                pattern: Pattern::Stream,
                score: tap.stream.score,
                objects: tap.stream.notes,
                segments: tap.stream.segments,
                peak: tap.stream.peak,
            },
            PatternScore {
                pattern: Pattern::Burst,
                score: tap.burst.score,
                objects: tap.burst.notes,
                segments: tap.burst.segments,
                peak: tap.burst.peak,
            },
            PatternScore {
                pattern: Pattern::Slider,
                score: slider.score,
                objects: slider.count,
                segments: slider.count,
                peak: slider.peak,
            },
            PatternScore {
                pattern: Pattern::Tech,
                score: tech.analysis.score,
                objects: tech.objects,
                segments: tech.segments,
                peak: tech.analysis.peak,
            },
        ];
        patterns.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| left.pattern.cmp(&right.pattern))
        });
        let primary = patterns
            .first()
            .filter(|entry| entry.score >= self.config.primary_score_min)
            .map(|entry| entry.pattern);

        Ok(MapAnalysis {
            primary,
            patterns,
            jump,
            stream: tap.stream,
            burst: tap.burst,
            slider,
            tech: tech.analysis,
            object_count: features.objects.len(),
            tap_object_count: features.circle_count,
            duration_ms: features.duration_ms,
        })
    }
}
