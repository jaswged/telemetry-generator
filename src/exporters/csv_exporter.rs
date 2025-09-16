use crate::models::TelemetryDataset;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use tracing::info;

pub struct CsvMetadataExporter;

impl CsvMetadataExporter {
    // Export telemetry meta data around run

    pub fn export(dataset: &TelemetryDataset, output_name: &str) -> Result<()> {
        info!("Inside export csv function");

        // Create the file first
        let csv_file = format!("output/{output_name}.metadata.csv");
        info!("Writing file to: {}", csv_file);
        let mut output_file: File = File::create(&csv_file)
            .with_context(|| format!("Failed to create the file yo! {}", &csv_file))?;

        // Write the header
        writeln!(
            output_file,
            "launch_id,launch_time,time_since_launch_ms,vehicle_type,engine_type,sample_rate_hz"
        )?;

        // Only 1 row to write
        if let Some(first) = dataset.readings.first() {
            writeln!(
                output_file,
                "{},{},{},{},{},{}",
                "id_1",
                dataset.launch_time,
                first.time_since_launch_ms,
                "Kerbal",
                "Narwhal",
                "todo:pass_me_in_sir",
            )?;
        }

        info!("Csv file write completed to {}", csv_file);
        Ok(())
    }
}
