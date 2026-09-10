use hdrhistogram::Histogram;
use serde::Serialize;

pub const LATENCY_MAX_NS: u64 = 60_000_000_000;

pub struct Metrics {
    latency: Histogram<u64>,
    overflow: u64,
    batches: [u64; 65],
}

#[derive(Debug, Serialize)]
pub struct MetricsSummary {
    pub latency_observations: u64,
    pub latency_recorded: u64,
    pub latency_overflow: u64,
    pub latency_max_trackable_ns: u64,
    pub latency_significant_digits: u8,
    pub latency_p50_ns: Option<u64>,
    pub latency_p95_ns: Option<u64>,
    pub latency_p99_ns: Option<u64>,
    /// Index is actual processing batch size; bucket zero is always empty.
    pub actual_batch_sizes: Vec<u64>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            latency: Histogram::new_with_bounds(1, LATENCY_MAX_NS, 3)
                .expect("fixed histogram bounds"),
            overflow: 0,
            batches: [0; 65],
        }
    }
}

impl Metrics {
    pub fn merge(&mut self, other: &Self) {
        self.latency
            .add(&other.latency)
            .expect("identical bounded histograms");
        self.overflow += other.overflow;
        for (total, count) in self.batches.iter_mut().zip(other.batches) {
            *total += count;
        }
    }
    pub fn record_latency(&mut self, ns: u64) {
        if ns > LATENCY_MAX_NS {
            self.overflow += 1;
        } else {
            self.latency.record(ns).expect("in-range latency");
        }
    }
    pub fn record_batch(&mut self, count: usize) {
        assert!((1..=64).contains(&count));
        self.batches[count] += 1;
    }
    pub fn summary(&self) -> MetricsSummary {
        let quantile = |q| (!self.latency.is_empty()).then(|| self.latency.value_at_quantile(q));
        MetricsSummary {
            latency_observations: self.latency.len() + self.overflow,
            latency_recorded: self.latency.len(),
            latency_overflow: self.overflow,
            latency_max_trackable_ns: LATENCY_MAX_NS,
            latency_significant_digits: 3,
            latency_p50_ns: quantile(0.5),
            latency_p95_ns: quantile(0.95),
            latency_p99_ns: quantile(0.99),
            actual_batch_sizes: self.batches.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn merging_preserves_counts_overflow_and_actual_batches() {
        let mut first = Metrics::default();
        first.record_latency(100);
        first.record_batch(1);
        let mut second = Metrics::default();
        second.record_latency(200);
        second.record_latency(LATENCY_MAX_NS + 1);
        second.record_batch(2);
        first.merge(&second);
        let summary = first.summary();
        assert_eq!(summary.latency_observations, 3);
        assert_eq!(summary.latency_recorded, 2);
        assert_eq!(summary.latency_overflow, 1);
        assert_eq!(summary.actual_batch_sizes[1], 1);
        assert_eq!(summary.actual_batch_sizes[2], 1);
        assert_eq!(summary.latency_p99_ns, Some(200));
    }
    #[test]
    fn empty_and_overflow_have_explicit_coverage() {
        let mut metrics = Metrics::default();
        assert_eq!(metrics.summary().latency_p50_ns, None);
        metrics.record_latency(LATENCY_MAX_NS + 1);
        assert_eq!(metrics.summary().latency_p99_ns, None);
        metrics.record_latency(0);
        metrics.record_latency(LATENCY_MAX_NS);
        let summary = metrics.summary();
        assert_eq!(summary.latency_observations, 3);
        assert_eq!(summary.latency_overflow, 1);
        assert_eq!(summary.latency_recorded, 2);
    }
}
