use crate::{
    features::{FeatureSet, ObjectKind},
    patterns::peak_for_objects,
    AnalysisConfig, TechAnalysis,
};

pub(crate) struct TechDetection {
    pub(crate) analysis: TechAnalysis,
    pub(crate) objects: usize,
    pub(crate) segments: usize,
}

pub(crate) fn detect(features: &FeatureSet, config: &AnalysisConfig) -> TechDetection {
    let mut rhythm_changes = 0;
    let mut rhythm_comparisons = 0;
    let mut complex_angles = 0;
    let mut measured_angles = 0;
    let mut object_indices = Vec::new();

    for pair in features.transitions.windows(2) {
        let [previous, current] = pair else {
            unreachable!();
        };

        if previous.to != current.from {
            continue;
        }

        rhythm_comparisons += 1;

        if quantized_rhythm(previous.elapsed_beats) != quantized_rhythm(current.elapsed_beats) {
            rhythm_changes += 1;
            object_indices.push(current.from);
            object_indices.push(current.to);
        }
    }

    for transition in &features.transitions {
        let Some(angle) = transition.angle_degrees else {
            continue;
        };

        measured_angles += 1;

        if (25.0..=155.0).contains(&angle) {
            complex_angles += 1;
            object_indices.push(transition.from);
            object_indices.push(transition.to);
        }
    }

    let slider_indices = features
        .objects
        .iter()
        .enumerate()
        .filter(|(_, object)| object.kind == ObjectKind::Slider)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    object_indices.extend(slider_indices.iter().copied());

    let slider_velocity_changes = slider_indices
        .windows(2)
        .filter(|pair| {
            let left = features.objects[pair[0]].slider_velocity;
            let right = features.objects[pair[1]].slider_velocity;
            left.max(right) / left.min(right).max(f64::EPSILON) >= 1.25
        })
        .count();
    let rhythm_complexity = ratio(rhythm_changes, rhythm_comparisons);
    let angle_complexity = ratio(complex_angles, measured_angles);
    let slider_share = ratio(slider_indices.len(), features.objects.len());
    let velocity_complexity = ratio(
        slider_velocity_changes,
        slider_indices.len().saturating_sub(1),
    );
    let score = (rhythm_complexity * 0.35
        + angle_complexity * 0.3
        + slider_share * 0.2
        + velocity_complexity * 0.15)
        .clamp(0.0, 1.0);
    object_indices.sort_unstable();
    object_indices.dedup();
    let objects = object_indices.len();
    let segments = rhythm_changes + complex_angles + slider_velocity_changes;

    TechDetection {
        analysis: TechAnalysis {
            score,
            rhythm_complexity,
            angle_complexity,
            slider_velocity_changes,
            peak: peak_for_objects(features, &object_indices, config),
        },
        objects,
        segments,
    }
}

fn quantized_rhythm(beats: f64) -> usize {
    const RHYTHMS: [f64; 10] = [
        0.125,
        1.0 / 6.0,
        0.25,
        1.0 / 3.0,
        0.5,
        0.75,
        1.0,
        1.5,
        2.0,
        4.0,
    ];

    RHYTHMS
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| (beats - **left).abs().total_cmp(&(beats - **right).abs()))
        .map_or(0, |(index, _)| index)
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}
