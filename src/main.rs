use anyhow::Result;
use chrono::Utc;
use clap::{Parser, Subcommand};
use num_format::{Locale, ToFormattedString};
use std::path::PathBuf;
use std::time::Instant;
use tracing::{Level, debug, info, warn};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod exporters;
mod generators;
mod models;

use crate::exporters::{CsvMetadataExporter, InfluxDBConfig, InfluxDBExporter, ParquetExporter};
use crate::generators::TelemetryGenerator;
use crate::models::{SensorEnum, TelemetryConfig, TelemetryDataset};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Setup logger
    // let _guard = init_logger(cli.log_level, cli.log_dir);
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "telemetry_generator=info".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_thread_ids(true)
                .with_thread_names(true),
        )
        .init();

    info!("Starting telemetry generator...");

    info!("Parsed CLI arguments");
    debug!("All cli: {:?}", cli);
    info!("Command: {:?}", cli.command);

    match &cli.command {
        Commands::Generate {
            duration,
            khz,
            launch_id,
            seed,
            disable_progress,
            max_rows,
            timestamp_jitter,
        } => {
            info!("Generating telemetry data...");
            let _ = generate_to_parquet(
                *duration,
                (*khz * 1000.0).round() as usize,
                launch_id, // other run details. vehicle type, engine type, etc.
                *seed,
                *disable_progress,
                *max_rows, // pass as Option<usize>
                *timestamp_jitter,
            );
            // Call the generate function from the generate module
            // if let Err(e) = telemetry_generator::generate::generate_telemetry(
            //     *duration,
            //     *sample_rate_khz,
            //     launch_id,
            //     *seed,
            //     *no_progress,
            //     *max_rows,
            //     *timestamp_jitter,
            // ) {
            //     error!("Error generating telemetry data: {:?}", e);
            // }
        }
        Commands::InfluxDB {
            url,
            token,
            org,
            bucket,
            batch_size,
        } => {
            info!("Sending data to InfluxDB at {}", url);
            info!("Sending data to InfluxDB bucket {}", bucket);
            info!("InfluxDB batch size {}", batch_size);
            debug!("Token: {}", token);

            let influx_exporter = InfluxDBExporter::new(InfluxDBConfig {
                url: url.clone(), //: url.take(),
                token: token.clone(),
                org: org.clone(),
                bucket: bucket.clone(),
                batch_size: *batch_size,
            });

            info!("Calling into influx generator");
            let dataset = TelemetryDataset {
                readings: Vec::new(),
                config: TelemetryConfig::default(),
                launch_time: Utc::now(),
            };
            let ret = influx_exporter.export(&dataset).await;

            // // Call the function to send data to InfluxDB
            // if let Err(e) =
            //     telemetry_generator::influxdb::send_to_influxdb(url, token, bucket, batch_size)
            // {
            //     error!("Error sending data to InfluxDB: {:?}", e);
            // }
        }
        Commands::Start => {
            info!("Starting server...");
            // Call the start server function
        }
        Commands::Stop => {
            info!("Stopping server...");
            // Call the stop server function
        }
        Commands::Status => {
            info!("Checking server status...");
            // Call the status function
        }
    }
    info!("Process ending...");
}

fn generate_to_parquet(
    duration: usize,
    sample_rate_hz: usize,
    launch_id: &str,
    seed: u64,
    disable_progress: bool,
    max_rows: Option<usize>,
    timestamp_jitter: f64,
) -> Result<()> {
    info!("Inside generate_to_parquet fn");
    let start_time = Instant::now();

    info!("Number of sensors: {}", SensorEnum::number_of_sensors());
    info!(
        "Hz to run sim at: {}",
        sample_rate_hz.to_formatted_string(&Locale::en)
    );
    info!("Duration of the test run: {}", duration);

    // Warn if sample rate is too high and would create too many rows for max_rows
    let estimated_points: usize = duration * sample_rate_hz * SensorEnum::number_of_sensors();
    info!(
        "Estimated number of data-points: {}",
        estimated_points.to_formatted_string(&Locale::en)
    );
    if max_rows.is_some() && estimated_points > max_rows.unwrap() {
        warn!(
            "Estimated points ({}) exceed max rows ({}). Consider increasing max rows or decreasing sample rate/duration.",
            estimated_points,
            max_rows.unwrap()
        );
    }

    // Setup telemetry generation
    let config: TelemetryConfig = TelemetryConfig {
        duration,
        sample_rate_hz,
        launch_id: launch_id.to_string(),
        seed,
        // disable_progress,
        max_rows,
        timestamp_jitter,
    };

    let mut generator = TelemetryGenerator::new(config);
    let dataset: TelemetryDataset = generator.generate(disable_progress);

    // Debug output here...

    // Write to Parquet
    // Todo geneate output file name from params. OR concatenate onto provided name. Make it optional if not already
    let output_file = format!("{launch_id}_{sample_rate_hz}hz_{duration}s"); //craft_file_name_parquet(config);
    ParquetExporter::export(&dataset, &output_file)?;

    // Save metadata to CSV
    info!("Write out metadata around the run");
    CsvMetadataExporter::export(&dataset, &output_file)?;

    let elapsed = start_time.elapsed();
    info!("Generation completed in {:.2?}s", elapsed.as_secs_f64());
    info!(
        "Generated {} readings",
        dataset.readings.len().to_formatted_string(&Locale::en)
    );

    Ok(())
}

#[derive(Parser, Debug)]
#[command(name = "Telemetry Generator")]
#[command(about = "A tool to generate mock telemetry data", long_about = None)]
#[command(author = "Jason Gedamke")]
struct Cli {
    #[arg(long, value_name = "LEVEL")]
    log_level: Option<Level>,

    // Location to save the log files
    #[arg(long, value_name = "DIRECTORY")]
    log_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the server
    Generate {
        // Duration of simulated flight in seconds
        #[arg(short, long, value_name = "DURATION", default_value = "120")]
        duration: usize,

        // Frequency rate. Default is 1 kHz = 1,000 Hz
        #[arg(long, value_name = "FREQUENCY", default_value = "1")]
        khz: f64,

        // TODO: Could also add other meta data. vehicle_type, engine_type, etc.
        #[arg(long, default_value = "SIM-001")]
        launch_id: String,

        #[arg(long, default_value = "1337")]
        seed: u64,

        // Disable progress bar
        #[arg(long, default_value = "false")]
        disable_progress: bool,

        #[arg(long)]
        max_rows: Option<usize>,

        #[arg(long, default_value = "50.0")]
        timestamp_jitter: f64,
    },
    // Generate data to send to InfluxDB
    // todo reuse some params from above in generate
    InfluxDB {
        #[arg(long, default_value = "http://localhost:8086")]
        url: String,
        #[arg(short, long)]
        token: String,
        #[arg(short, long)]
        org: String,
        #[arg(short, long)]
        bucket: String,
        #[arg(long, default_value = "5000")]
        batch_size: usize,
    },
    // Todo idea: Generate data nonstop and feed into a local InfluxDB instance
    // Use it to test out theories for data storage
    Start,
    // Stop the server
    Stop,
    // Check the server status
    Status,
}

// // fn init_logger(log_level: Option<Level>, log_dir: Option<&Path>) -> Option<WorkerGuard> {
//     let _level = log_level.unwrap_or(Level::INFO);

//     let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
// }
