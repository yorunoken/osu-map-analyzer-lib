use crate::{
    features::{FeatureSet, ObjectKind},
    patterns::peak_for_objects,
    AnalysisConfig, SliderAnalysis,
};

pub(crate) fn detect(features: &FeatureSet, config: &AnalysisConfig) -> SliderAnalysis {
    let mut count = 0;
    let mut repeats = 0;
    let mut travel_distance = 0.0;
    let mut total_duration = 0.0;
    let mut object_indices = Vec::new();

    for (index, object) in features.objects.iter().enumerate() {
        if object.kind != ObjectKind::Slider {
            continue;
        }

        count += 1;
        repeats += object.slider_repeats;
        travel_distance += object.slider_travel;
        total_duration += object.slider_duration;
        object_indices.push(index);
    }

    let object_share = count as f64 / features.objects.len().max(1) as f64;
    let duration_ratio = (total_duration / features.duration_ms.max(1.0)).clamp(0.0, 1.0);
    let repeat_strength = (repeats as f64 / (count.max(1) * 2) as f64).clamp(0.0, 1.0);
    let travel_strength =
        (travel_distance / (features.objects.len().max(1) * 4) as f64).clamp(0.0, 1.0);

    SliderAnalysis {
        score: (object_share * 0.5
            + duration_ratio * 0.3
            + repeat_strength * 0.1
            + travel_strength * 0.1)
            .clamp(0.0, 1.0),
        count,
        repeats,
        travel_distance,
        duration_ratio,
        peak: peak_for_objects(features, &object_indices, config),
    }
}
