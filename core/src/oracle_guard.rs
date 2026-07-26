use chrono::{DateTime, Duration, Utc};

pub struct OracleReading {
    pub price: f64,
    pub source: String,
    pub timestamp: DateTime<Utc>,
}

pub struct OracleGuard {
    max_age: Duration,
    min_sources: usize,
    max_deviation_pct: f64,
}

impl OracleGuard {
    pub fn new(max_age_secs: i64, min_sources: usize, max_deviation_pct: f64) -> Self {
        Self {
            max_age: Duration::seconds(max_age_secs),
            min_sources,
            max_deviation_pct,
        }
    }

    pub fn validate_freshness(&self, reading: &OracleReading) -> bool {
        let age = Utc::now() - reading.timestamp;
        age <= self.max_age
    }

    pub fn validate_multi_source(&self, readings: &[OracleReading]) -> Option<f64> {
        let fresh: Vec<f64> = readings
            .iter()
            .filter(|r| self.validate_freshness(r))
            .map(|r| r.price)
            .collect();

        if fresh.len() < self.min_sources {
            return None;
        }

        let mut sorted = fresh.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            Some((sorted[mid - 1] + sorted[mid]) / 2.0)
        } else {
            Some(sorted[mid])
        }
    }

    pub fn validate_deviation(&self, readings: &[OracleReading]) -> bool {
        if readings.len() < 2 {
            return true;
        }

        let prices: Vec<f64> = readings.iter().map(|r| r.price).collect();
        let min = prices.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        if min == 0.0 {
            return false;
        }

        let deviation = ((max - min) / min) * 100.0;
        deviation <= self.max_deviation_pct
    }
}
