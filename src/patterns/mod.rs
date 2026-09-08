use crate::{features::FeatureSet, AnalysisConfig, Peak};

pub(crate) mod jump;
pub(crate) mod slider;
pub(crate) mod tap;
pub(crate) mod tech;

pub(crate) fn peak_for_objects(
    features: &FeatureSet,
    object_indices: &[usize],
    config: &AnalysisConfig,
) -> Option<Peak> {
    if object_indices.is_empty() {
        return None;
    }

    let mut counts = vec![0_usize; features.section_object_counts.len()];
    let mut seen = vec![false; features.objects.len()];

    for &object_index in object_indices {
        if seen.get(object_index).copied().unwrap_or(true) {
            continue;
        }

        seen[object_index] = true;
        let time = features.objects[object_index].start_time;
        let section =
            ((time - features.start_time).max(0.0) / config.peak_window_ms).floor() as usize;

        if let Some(count) = counts.get_mut(section) {
            *count += 1;
        }
    }

    counts
        .iter()
        .enumerate()
        .filter(|(_, count)| **count > 0)
        .map(|(section, &count)| {
            let total = features.section_object_counts[section].max(1);
            Peak {
                start_time_ms: features.start_time + section as f64 * config.peak_window_ms,
                score: (count as f64 / total as f64).clamp(0.0, 1.0),
            }
        })
        .max_by(|left, right| {
            left.score
                .total_cmp(&right.score)
                .then_with(|| right.start_time_ms.total_cmp(&left.start_time_ms))
        })
}
