pub(crate) fn mean(values: &[f64]) -> f64 {
    safe_div(
        values
            .iter()
            .copied()
            .filter(|value| value.is_finite())
            .sum(),
        valid_count(values) as f64,
    )
}

pub(crate) fn standard_deviation(values: &[f64]) -> f64 {
    let average = mean(values);
    let finite: Vec<_> = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .collect();
    safe_div(
        finite.iter().map(|value| (value - average).powi(2)).sum(),
        finite.len() as f64,
    )
    .sqrt()
}

pub(crate) fn min(values: &[f64]) -> f64 {
    values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .reduce(f64::min)
        .unwrap_or(0.0)
}

pub(crate) fn max(values: &[f64]) -> f64 {
    values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .reduce(f64::max)
        .unwrap_or(0.0)
}

/// Returns a nearest-rank percentile after sorting finite values.
pub(crate) fn percentile(values: &[f64], percentile: f64) -> f64 {
    let mut sorted: Vec<_> = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .collect();
    if sorted.is_empty() {
        return 0.0;
    }

    sorted.sort_by(f64::total_cmp);
    let percentile = percentile.clamp(0.0, 1.0);
    sorted[((sorted.len() - 1) as f64 * percentile).round() as usize]
}

pub(crate) fn ratio(value: usize, total: usize) -> f64 {
    safe_div(value as f64, total as f64)
}

pub(crate) fn safe_div(value: f64, divisor: f64) -> f64 {
    if value.is_finite() && divisor.is_finite() && divisor > 0.0 {
        value / divisor
    } else {
        0.0
    }
}

fn valid_count(values: &[f64]) -> usize {
    values.iter().filter(|value| value.is_finite()).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statistics_are_safe_for_empty_and_non_finite_inputs() {
        assert_eq!(mean(&[]), 0.0);
        assert_eq!(standard_deviation(&[f64::NAN]), 0.0);
        assert_eq!(min(&[]), 0.0);
        assert_eq!(max(&[]), 0.0);
        assert_eq!(percentile(&[], 0.95), 0.0);
        assert_eq!(safe_div(1.0, 0.0), 0.0);
        assert_eq!(ratio(1, 0), 0.0);
    }

    #[test]
    fn percentiles_sort_and_select_deterministically() {
        let values = [4.0, 1.0, 3.0, 2.0];
        assert_eq!(percentile(&values, 0.50), 3.0);
        assert_eq!(percentile(&values, 0.75), 3.0);
        assert_eq!(percentile(&values, 0.90), 4.0);
        assert_eq!(percentile(&values, 0.95), 4.0);
    }
}
