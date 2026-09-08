use crate::{
    features::{FeatureSet, Transition},
    patterns::peak_for_objects,
    AnalysisConfig, JumpAnalysis,
};

pub(crate) fn detect(
    features: &FeatureSet,
    stream_edges: &[bool],
    config: &AnalysisConfig,
) -> JumpAnalysis {
    let mut accumulator = Accumulator::default();
    let mut run: Vec<usize> = Vec::new();

    for (index, transition) in features.transitions.iter().enumerate() {
        let jump =
            !stream_edges.get(index).copied().unwrap_or(false) && is_jump(transition, config);
        let continuous = run.last().is_none_or(|&previous_index| {
            features.transitions[previous_index].to == transition.from
        });

        if jump && continuous {
            run.push(index);
        } else {
            accumulator.push_run(&run, features);
            run.clear();

            if jump {
                run.push(index);
            }
        }
    }

    accumulator.push_run(&run, features);
    accumulator.finish(features, config)
}

fn is_jump(transition: &Transition, config: &AnalysisConfig) -> bool {
    transition.normalized_distance >= config.jump_min_distance
        && transition.delta_ms <= config.jump_max_interval_ms
        && transition.elapsed_beats <= config.jump_max_interval_beats
}

#[derive(Default)]
struct Accumulator {
    segments: usize,
    notes: usize,
    longest: usize,
    distance_total: f64,
    max_distance: f64,
    edge_count: usize,
    object_indices: Vec<usize>,
}

impl Accumulator {
    fn push_run(&mut self, run: &[usize], features: &FeatureSet) {
        if run.is_empty() {
            return;
        }

        let note_count = run.len() + 1;
        self.segments += 1;
        self.notes += note_count;
        self.longest = self.longest.max(note_count);
        self.edge_count += run.len();
        self.object_indices.push(features.transitions[run[0]].from);

        for &edge in run {
            let transition = features.transitions[edge];
            self.distance_total += transition.normalized_distance;
            self.max_distance = self.max_distance.max(transition.normalized_distance);
            self.object_indices.push(transition.to);
        }
    }

    fn finish(self, features: &FeatureSet, config: &AnalysisConfig) -> JumpAnalysis {
        let average_distance = if self.edge_count == 0 {
            0.0
        } else {
            self.distance_total / self.edge_count as f64
        };
        let density = self.notes as f64 / features.objects.len().max(1) as f64;
        let distance_strength =
            ((average_distance / config.jump_min_distance) - 1.0).clamp(0.0, 1.0);
        let length_strength = (self.longest as f64 / 12.0).clamp(0.0, 1.0);

        JumpAnalysis {
            score: (density * 0.6 + distance_strength * 0.25 + length_strength * 0.15)
                .clamp(0.0, 1.0),
            segments: self.segments,
            notes: self.notes,
            longest: self.longest,
            average_distance,
            max_distance: self.max_distance,
            peak: peak_for_objects(features, &self.object_indices, config),
        }
    }
}
