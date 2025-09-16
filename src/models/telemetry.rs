use super::sensor::{SensorEnum, SensorValue};
use chrono::{DateTime, Utc};
use rand::Rng;
use rand_distr::{Distribution, Normal};
use tracing::info;

#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub duration: usize,
    pub sample_rate_hz: usize,
    pub launch_id: String,
    pub seed: u64,
    pub max_rows: Option<usize>,
    pub timestamp_jitter: f64,
}

impl TelemetryConfig {
    pub fn get_total_points(&self) -> usize {
        let total_points = self.duration * self.sample_rate_hz * SensorEnum::number_of_sensors();

        if let Some(max) = self.max_rows {
            std::cmp::min(total_points, max)
        } else {
            total_points
        }
    }

    pub fn get_total_readings(&self) -> usize {
        self.duration * self.sample_rate_hz
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        TelemetryConfig {
            duration: 120,          // 2 minutes
            sample_rate_hz: 10_000, // 10 kHz
            launch_id: "eg_launch".into(),
            seed: 1337,
            max_rows: None,
            timestamp_jitter: 25.0, // 25 microseconds
        }
    }
}

pub struct TimestampJitter {
    distribution: Normal<f64>,
}

impl TimestampJitter {
    pub fn new(std_dev_us: f64) -> Self {
        Self {
            distribution: Normal::new(0.0, std_dev_us).unwrap(),
        }
    }
    pub fn apply<R: Rng>(&self, timestamp: DateTime<Utc>, rng: &mut R) -> DateTime<Utc> {
        let jitter_micros = self.distribution.sample(rng).round() as i64;

        // Add jitter to provided timestamp
        timestamp + chrono::Duration::microseconds(jitter_micros)
    }
}

#[derive(Debug)]
pub struct TelemetryDataset {
    pub readings: Vec<TelemetryReading>,
    pub config: TelemetryConfig,
    pub launch_time: DateTime<Utc>,
    // pub base_timestamps: Vec<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct TelemetryReading {
    pub timestamp: DateTime<Utc>,
    pub time_since_launch_ms: u64,
    pub sensor: SensorEnum,
    pub value: SensorValue,
    // Todo InfluxDb tags
}

impl TelemetryReading {
    pub fn new(
        timestamp: DateTime<Utc>,
        time_since_launch_ms: u64,
        sensor: SensorEnum,
        value: SensorValue,
    ) -> Self {
        Self {
            timestamp,
            time_since_launch_ms,
            sensor,
            value,
        }
    }
    pub fn to_line_protocol(&self, measurement: &str) -> String {
        info!("Measurement is: {}. at ts: {}", measurement, self.timestamp);
        // let tags = format!("sensor_type={}", self.sensor_type.field_name());
        "todo".to_string()
    }
}
