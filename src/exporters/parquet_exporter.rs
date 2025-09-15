use crate::models::{SensorValue, TelemetryDataset};
use anyhow::{Context, Result};
use arrow::array::{ArrayRef, Float64Array, StringArray, TimestampMicrosecondArray};
use arrow::record_batch::RecordBatch;
use arrow_array::UInt64Array;
use arrow_schema::{DataType, Field, Schema};
use indicatif::{ProgressBar, ProgressStyle};
use parquet::arrow::arrow_writer::ArrowWriter;
use parquet::file::properties::WriterProperties;
use std::{fs::File, sync::Arc};
use tracing::{info, warn};

pub struct ParquetExporter;

impl ParquetExporter {
    // pub fn new() -> Self {
    //     ParquetExporter {}
    // }

    pub fn export(
        dataset: &TelemetryDataset,
        file_path: &str,
        disable_progress: bool,
    ) -> Result<()> {
        info!("Inside export parquet");

        // Don't write anything out...
        if dataset.readings.is_empty() {
            warn!("No readings to export. Exiting export.");
            return Ok(()); // todo return something else.
        }

        let schema: Schema = Self::create_schema();

        let output_file: File = File::create(format!("{file_path}.parquet"))
            .with_context(|| format!("Failed to create output file at {file_path}"))?;

        // Create arrow writer
        let props = WriterProperties::builder()
            .set_compression(parquet::basic::Compression::SNAPPY)
            .build();
        let mut writer: ArrowWriter<File> =
            ArrowWriter::try_new(output_file, Arc::new(schema.clone()), Some(props))
                .context("Failed to create arrow writer")?;

        let batch: RecordBatch = Self::convert_to_record_batch(dataset, schema)?;

        // Write to file
        writer
            .write(&batch)
            .with_context(|| "Failed to write record batch to Parquet")?;

        writer
            .close()
            .with_context(|| "Failed to close Parquet writer")?;

        // Implement Parquet export logic here
        info!(
            "Exporting {} readings to Parquet file at {}",
            batch.num_rows(),
            file_path
        );

        Ok(())
    }

    fn create_schema() -> Schema {
        Schema::new(vec![
            Field::new(
                "timestamp",
                DataType::Timestamp(arrow::datatypes::TimeUnit::Microsecond, None), // todo is Nano second possible?
                false,
            ),
            Field::new("time_since_launch_ms", DataType::UInt64, false),
            Field::new("sensor_type", DataType::Utf8, false),
            Field::new("value", DataType::Float64, false), // was 3 columns for Float, I64, U64
        ])
    }

    // Convert telemetry record to arrow record batch
    fn convert_to_record_batch(dataset: &TelemetryDataset, schema: Schema) -> Result<RecordBatch> {
        info!("Inside convert to record batch");
        let total_readings = dataset.readings.len();
        // todo currently no choice on the PB
        let pb = ProgressBar::new(total_readings as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:50.cyan/blue}] {pos:>7}/{len:7} readings ({percent}%) {msg} ({eta})")?
                .progress_chars("#>-"),
        );

        // prepare arrays
        let mut timestamps = Vec::with_capacity(total_readings);
        let mut time_since_launch_ms = Vec::with_capacity(total_readings);
        let mut sensor_types = Vec::with_capacity(total_readings);
        let mut values = Vec::with_capacity(total_readings);

        // Fill arrays from readings
        for (i, reading) in dataset.readings.iter().enumerate() {
            if i % 100 == 0 {
                pb.set_position(i as u64);
            }

            timestamps.push(reading.timestamp.timestamp_micros());
            time_since_launch_ms.push(reading.time_since_launch_ms);
            sensor_types.push(reading.sensor.field_name().to_string());

            values.push(match &reading.value {
                SensorValue::Float(v) => *v, // as f64,
                // SensorValue::Int(v) => *v as f64,
                // SensorValue::UnsignedInt(v) => *v as f64,
                SensorValue::String(v) => todo!("Can't pass a string here: {v}. need to refactor"),
                // SensorValue::State(v) => todo!(),
                // SensorValue::Status(v) => todo!(),
            });
        }

        pb.finish_with_message("Arrow conversion complete");

        // Create Arrays from collected values
        let arrays: Vec<ArrayRef> = vec![
            Arc::new(TimestampMicrosecondArray::from(timestamps)),
            Arc::new(UInt64Array::from(time_since_launch_ms)),
            Arc::new(StringArray::from(sensor_types)),
            Arc::new(Float64Array::from(values)),
            // value ints, uInts
        ];

        let batch = RecordBatch::try_new(Arc::new(schema), arrays)
            .with_context(|| "Failed to create RecordBatch from arrays")?;
        info!("Successfully created Arrow RecordBatch");

        Ok(batch)
    }
}
