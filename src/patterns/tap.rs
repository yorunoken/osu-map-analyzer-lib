use crate::{
    features::{FeatureSet, ObjectKind, Transition},
    patterns::peak_for_objects,
    AnalysisConfig, BurstAnalysis, StreamAnalysis,
};

pub(crate) struct TapDetection {
    pub(crate) stream: StreamAnalysis,
    pub(crate) burst: BurstAnalysis,
    pub(crate) stream_edges: Vec<bool>,
}

pub(crate) fn detect(features: &FeatureSet, config: &AnalysisConfig) -> TapDetection {
    let mut stream = Accumulator::default();
    let mut burst = Accumulator::default();
    let mut stream_edges = vec![false; features.transitions.len()];
    let mut run: Vec<usize> = Vec::new();

    for (index, transition) in features.transitions.iter().enumerate() {
        let fast = is_fast(features, transition, config);
        let continuous = run.last().is_none_or(|&previous_index| {
            features.transitions[previous_index].to == transition.from
        });
        let rhythm_matches = run.last().is_none_or(|&previous_index| {
            let previous = features.transitions[previous_index].elapsed_beats;
            (transition.elapsed_beats - previous).abs() / previous.max(f64::EPSILON)
                <= config.rhythm_tolerance
        });

        if fast && continuous && rhythm_matches {
            run.push(index);
        } else {
            finish_run(
                &mut run,
                features,
                config,
                &mut stream,
                &mut burst,
                &mut stream_edges,
            );

            if fast {
                run.push(index);
            }
        }
    }

    finish_run(
        &mut run,
        features,
        config,
        &mut stream,
        &mut burst,
        &mut stream_edges,
    );

    TapDetection {
        stream: stream.into_stream(features, config),
        burst: burst.into_burst(features, config),
        stream_edges,
    }
}

fn is_fast(features: &FeatureSet, transition: &Transition, config: &AnalysisConfig) -> bool {
    let from = features.objects[transition.from];
    let to = features.objects[transition.to];

    from.kind == ObjectKind::Circle
        && to.kind == ObjectKind::Circle
        && transition.delta_ms <= config.fast_interval_ms
        && transition.elapsed_beats <= config.fast_interval_beats
}

fn finish_run(
    run: &mut Vec<usize>,
    features: &FeatureSet,
    config: &AnalysisConfig,
    stream: &mut Accumulator,
    burst: &mut Accumulator,
    stream_edges: &mut [bool],
) {
    if run.is_empty() {
        return;
    }

    let note_count = run.len() + 1;

    if note_count >= config.stream_min_notes {
        stream.push(run, features);

        for &edge in run.iter() {
            stream_edges[edge] = true;
        }
    } else if note_count >= 3 {
        burst.push(run, features);
    }

    run.clear();
}

#[derive(Default)]
struct Accumulator {
    segments: usize,
    notes: usize,
    longest: usize,
    total_interval_ms: f64,
    edge_count: usize,
    object_indices: Vec<usize>,
}

impl Accumulator {
    fn push(&mut self, run: &[usize], features: &FeatureSet) {
        self.segments += 1;
        self.notes += run.len() + 1;
        self.longest = self.longest.max(run.len() + 1);
        self.edge_count += run.len();

        let first = features.transitions[run[0]].from;
        self.object_indices.push(first);

        for &edge in run {
            let transition = features.transitions[edge];
            self.total_interval_ms += transition.delta_ms;
            self.object_indices.push(transition.to);
        }
    }

    fn into_stream(self, features: &FeatureSet, config: &AnalysisConfig) -> StreamAnalysis {
        let density = ratio(self.notes, features.circle_count);
        let length_strength =
            (self.longest as f64 / (config.stream_min_notes * 4) as f64).clamp(0.0, 1.0);

        StreamAnalysis {
            score: clamp_score(density * 0.75 + length_strength * 0.25),
            segments: self.segments,
            notes: self.notes,
            longest: self.longest,
            average_note_interval_ms: average(self.total_interval_ms, self.edge_count),
            peak: peak_for_objects(features, &self.object_indices, config),
        }
    }

    fn into_burst(self, features: &FeatureSet, config: &AnalysisConfig) -> BurstAnalysis {
        let density = ratio(self.notes, features.circle_count);
        let length_strength =
            (self.longest as f64 / (config.stream_min_notes - 1) as f64).clamp(0.0, 1.0);

        BurstAnalysis {
            score: clamp_score(density * 0.7 + length_strength * 0.3),
            segments: self.segments,
            notes: self.notes,
            longest: self.longest,
            average_note_interval_ms: average(self.total_interval_ms, self.edge_count),
            peak: peak_for_objects(features, &self.object_indices, config),
        }
    }
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    numerator as f64 / denominator.max(1) as f64
}

fn average(total: f64, count: usize) -> f64 {
    if count == 0 {
        0.0
    } else {
        total / count as f64
    }
}

fn clamp_score(score: f64) -> f64 {
    score.clamp(0.0, 1.0)
}
