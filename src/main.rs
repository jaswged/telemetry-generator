use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{Level, error, info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    println!("Hello, world!");
    let cli = Cli::parse();

    // Setup logger
    // let _guard = init_logger(cli.log_level, cli.log_dir);
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "telemetry_generator=info".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_thread_ids(true)
                .with_thread_names(true),
        )
        .init();

    info!("Starting telemetry generator...");

    info!("Parsed CLI arguments");
    info!("All cli: {:?}", cli);
    info!("Command: {:?}", cli.command);

    match &cli.command {
        Commands::Generate {
            output,
            duration,
            sample_rate_khz,
            launch_id,
            seed,
            no_progress,
            max_rows,
            timestamp_jitter,
        } => {
            info!("Generating telemetry data...");
            // Call the generate function from the generate module
            // if let Err(e) = telemetry_generator::generate::generate_telemetry(
            //     output,
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
            bucket,
            batch_size,
        } => {
            info!("Sending data to InfluxDB at {}", url);
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

#[derive(Parser, Debug)]
#[command(name = "Telemetry Generator")]
#[command(about = "A tool to generate mock telemetry data", long_about = None)]
#[command(author = "Jason Gedamke")]
struct Cli {
    #[arg(long, value_name = "LEVEL")]
    log_level: Option<Level>,

    #[arg(long, value_name = "DIRECTORY")]
    log_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the server
    Generate {
        #[arg(short, long, value_name = "FILE")]
        output: PathBuf,

        // Duration of simulated flight in seconds
        #[arg(short, long, value_name = "DURATION", default_value = "180")]
        duration: f64,

        // Frequcny rate. Default is 10 kHz = 10,000 Hz
        #[arg(short, long, value_name = "FREQUENCY", default_value = "10")]
        sample_rate_khz: usize,

        // Could also add other meta data. vehicle_type, engine_type, etc.
        #[arg(long, default_value = "SIM-001")]
        launch_id: String,

        #[arg(long)]
        seed: Option<u64>,

        // Disable progress bar
        #[arg(long)]
        no_progress: bool,

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
        #[arg(long)]
        token: String,
        #[arg(short, long)]
        bucket: String,
        #[arg(long, default_value = "5000")]
        batch_size: String,
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
