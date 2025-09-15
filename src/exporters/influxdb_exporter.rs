use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use influxdb2::Client;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::models::TelemetryDataset;

#[derive(Debug, Clone)]
pub struct InfluxDBConfig {
    pub url: String,
    pub token: String,
    pub org: String,
    pub bucket: String,
    pub batch_size: usize,
}

impl Default for InfluxDBConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:8086".to_string(),
            token: "my_token".to_string(),
            org: "my_org".to_string(),
            bucket: "my_bucket".to_string(),
            batch_size: 5000,
        }
    }
}

#[derive(Debug)]
pub struct InfluxDBExporter {
    client: Client,
    config: InfluxDBConfig,
}

impl InfluxDBExporter {
    pub fn new(config: InfluxDBConfig) -> Self {
        let client = Client::new(&config.url, &config.org, &config.token);
        Self { client, config }
    }

    pub async fn export(&self, dataset: &TelemetryDataset) -> Result<()> {
        info!("inside export influx db function");

        if dataset.readings.is_empty() {
            warn!("No data detected to export!");
            return Ok(());
        }

        let total_readings = dataset.readings.len();
        let batch_count = (total_readings + self.config.batch_size - 1) / self.config.batch_size;

        let pb = ProgressBar::new(batch_count as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:50.cyan/blue}] {pos}/{len} batches ({percent}%) {msg}")?
                .progress_chars("#>-"),
        );
        pb.set_message(format!(
            "Sending data to Influx with BS: {}",
            self.config.batch_size
        ));

        for (batch_idx, chunk) in dataset.readings.chunks(self.config.batch_size).enumerate() {
            let mut line_data = String::new();

            for reading in chunk {
                let line = reading.to_line_protocol("rocket_telemetry");
                line_data.push_str(&line);
                line_data.push('\n');
            }

            // Write the batch to the server
            let write_result = self
                .client
                .write_line_protocol(&self.config.org, &self.config.bucket, line_data)
                .await;

            match write_result {
                Ok(_) => {
                    pb.set_position(batch_idx as u64 + 1);
                    pb.set_message(format!(
                        "Sent batch {}/{} ({} readings)",
                        batch_idx + 1,
                        batch_count,
                        chunk.len()
                    ));
                }
                Err(e) => {
                    error!(error = %e, batch_idx, "Failed to send batch to Influx");
                    return Err(anyhow::anyhow!("Influx DB write error!: {}", e));
                }
            }
        }

        pb.finish_with_message(format!(
            "Successfully exported to InfluxDB: {total_readings}"
        ));
        info!(
            total_readings,
            batch_count, "Successfully exported data to influxdb"
        );

        Ok(())
    }
}
