use crate::models::{
    SensorEnum, SensorValue, TelemetryConfig, TelemetryDataset, TelemetryReading, TimestampJitter,
};
use chrono::{DateTime, Duration, Utc};
use indicatif::{ProgressBar, ProgressStyle};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Normal};
use tracing::{error, info, instrument, warn};

pub struct TelemetryGenerator {
    config: TelemetryConfig,
    rng: StdRng,
}

impl TelemetryGenerator {
    #[instrument(skip(config), name = "TelemetryGenerator::new")]
    pub fn new(config: TelemetryConfig) -> Self {
        // todo seed from timestamp so its always different
        let random_seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        info!("Random seed would be: {}", random_seed);
        info!("Seeding RNG with {}", config.seed);
        let rng = StdRng::seed_from_u64(config.seed);
        Self { config, rng }
    }

    #[instrument(skip(self), name = "generate")]
    pub fn generate(&mut self, disable_progress: bool) -> TelemetryDataset {
        info!("Inside generate function");
        let launch_time = Utc::now();
        let total_readings: usize = self.config.get_total_readings();
        let sensors: usize = SensorEnum::number_of_sensors();
        let total_points: usize = total_readings * sensors;

        if total_points == 0 {
            warn!("No data points to generate! Check the configuration. Returning empty dataset.");
            return TelemetryDataset {
                readings: Vec::new(),
                config: self.config.clone(),
                launch_time,
                // base_timestamps: Vec::new(),
            };
        }

        // Setup Progress Bar option
        let progress: Option<ProgressBar> = if disable_progress {
            None
        } else {
            let pb = ProgressBar::new(total_points as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:50.cyan/blue}] {pos:>7}/{len:7} timestamps ({percent}%) {msg} ({eta})")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            Some(pb)
        };

        // Initialize the sim state. todo move to Struct itself. Jason
        let mut sim_state = SimulationState::initialize();

        // Generate all readings
        let mut all_readings: Vec<TelemetryReading> = Vec::with_capacity(total_points);
        let time_step_s = 1.0 / self.config.sample_rate_hz as f64;
        info!(
            "Time step size is: {:6.4} s or {:6.4} ms",
            time_step_s,
            time_step_s / 1000.0
        );
        info!("\n!Verify if you like the above formating dude!");

        // initialize noise distributions
        let pressure_noise = Normal::new(0.0, 1000.0).unwrap();
        let temperature_noise = Normal::new(0.0, 1.0).unwrap();
        let flow_rate_noise = Normal::new(0.0, 0.1).unwrap();
        let vibration_noise = Normal::new(0.0, 0.01).unwrap();
        let altitude_noise = Normal::new(0.0, 0.01).unwrap();

        // Create timestamp jitterer
        let timestamp_jitter = TimestampJitter::new(self.config.timestamp_jitter);

        // Store base timestamps for reference without jitter if needed
        // let mut base_timestamps: Vec<DateTime<Utc>> = Vec::with_capacity(total_points);

        // Loop through each sensor reading time
        for i in 0..total_readings {
            // Update progress bar every 1000 readings
            if let Some(pb) = &progress {
                if i % 1000 == 0 {
                    pb.set_position(i as u64);
                }
            }

            // Calculate base timestamp for this data point
            let base_timestamp_to_jitter: DateTime<Utc> =
                launch_time + Duration::milliseconds(sim_state.time_since_launch_ms as i64);
            // base_timestamps.push(base_timestamp);

            // Generate readings for all sensors with jittered timestamps
            let new_readings: Vec<TelemetryReading> = self.generate_readings_from_sim_state(
                &mut sim_state,
                base_timestamp_to_jitter,
                pressure_noise,
                temperature_noise,
                flow_rate_noise,
                vibration_noise,
                altitude_noise,
                &timestamp_jitter,
            );

            all_readings.extend(new_readings);

            // update simulation state for next iteration
            self.update_simulation_state(&mut sim_state, time_step_s, i, total_readings);

            // calculate precise millisecond time based on current step
            sim_state.time_since_launch_ms = (i as f64 * time_step_s * 1000.0).round() as u64;
        }

        // Finalize progress bar
        if let Some(pb) = progress {
            pb.finish_with_message("Data generation complete");
        }

        info!(
            "Telemetry dataset generated with {} readings",
            all_readings.len()
        );

        TelemetryDataset {
            readings: all_readings,
            config: self.config.clone(),
            launch_time,
            // base_timestamps,
        }
    }

    fn generate_readings_from_sim_state(
        &mut self,
        sim_state: &mut SimulationState,
        base_timestamp: DateTime<Utc>,
        pressure_noise: Normal<f64>,
        temperature_noise: Normal<f64>,
        flow_rate_noise: Normal<f64>,
        vibration_noise: Normal<f64>,
        altitude_noise: Normal<f64>,
        timestamp_jitter: &TimestampJitter,
    ) -> Vec<TelemetryReading> {
        // Todo: Too many lines here. Break into methods
        // For this simulation state we need to construct the telemetry records foreach sensor
        let mut readings: Vec<TelemetryReading> =
            Vec::with_capacity(SensorEnum::number_of_sensors());

        // Pre-sample all noise values, so we only borrow self.rng once
        let altitude_noise_val = altitude_noise.sample(&mut self.rng);
        let pressure_noise_val = pressure_noise.sample(&mut self.rng);
        let temperature_noise_val = temperature_noise.sample(&mut self.rng);
        let flow_rate_noise_val = flow_rate_noise.sample(&mut self.rng);
        let vibration_noise_val_x = vibration_noise.sample(&mut self.rng);
        let vibration_noise_val_y = vibration_noise.sample(&mut self.rng);
        let vibration_noise_val_z = vibration_noise.sample(&mut self.rng);

        let turbo_pump_rpm_noise = self.rng.gen_range(-50.0..50.0);
        let thrust_n_noise = self.rng.gen_range(-10.0..100.0);
        let specific_impulse_noise = self.rng.gen_range(-0.5..0.5);
        let nozzle_temperature_noise = temperature_noise.sample(&mut self.rng) * 2.0;
        let roll_angle_noise = self.rng.gen_range(-0.5..0.5);
        let pitch_angle_noise = self.rng.gen_range(-0.5..0.5);
        let yaw_angle_noise = self.rng.gen_range(-0.5..0.5);
        let vibration_freq_noise = self.rng.gen_range(-5.0..5.0);

        // Add readings foreach sensor type
        let sensor_values = vec![
            (
                SensorEnum::Acceleration,
                SensorValue::Float(sim_state.acceleration_mps2),
            ),
            (
                SensorEnum::Altitude,
                SensorValue::Float(sim_state.altitude_m + altitude_noise_val),
            ),
            (
                SensorEnum::Velocity,
                SensorValue::Float(sim_state.velocity_mps),
            ),
            (
                SensorEnum::ChamberPressure,
                SensorValue::Float(sim_state.chamber_pressure_pa + pressure_noise_val * 0.5),
            ),
            (
                SensorEnum::ChamberTemperature,
                SensorValue::Float(sim_state.chamber_temperature_k + temperature_noise_val * 0.2),
            ),
            (
                SensorEnum::OxidizerPressure,
                SensorValue::Float(sim_state.oxidizer_pressure_pa + pressure_noise_val * 0.5),
            ),
            (
                SensorEnum::OxidizerFlowRate,
                SensorValue::Float(sim_state.oxidizer_flow_rate_kgps + flow_rate_noise_val),
            ),
            (
                SensorEnum::OxidizerTemperature,
                SensorValue::Float(sim_state.oxidizer_temperature_k + temperature_noise_val * 0.2),
            ),
            (
                SensorEnum::FuelPressure,
                SensorValue::Float(sim_state.fuel_pressure_pa + pressure_noise_val * 0.5),
            ),
            (
                SensorEnum::FuelFlowRate,
                SensorValue::Float(sim_state.fuel_flow_rate_kgps + flow_rate_noise_val),
            ),
            (
                SensorEnum::FuelTemperature,
                SensorValue::Float(sim_state.fuel_temperature_k + temperature_noise_val),
            ),
            (
                SensorEnum::TurboPumpRpm,
                SensorValue::Float(sim_state.turbo_pump_rpm + turbo_pump_rpm_noise),
            ),
            (
                SensorEnum::Thrust,
                SensorValue::Float(sim_state.thrust_n + thrust_n_noise),
            ),
            (
                SensorEnum::SpecificImpulse,
                SensorValue::Float(sim_state.specific_impulse_s + specific_impulse_noise),
            ),
            (
                SensorEnum::NozzleTemperature,
                SensorValue::Float(sim_state.nozzle_temperature_k + nozzle_temperature_noise),
            ),
            (
                SensorEnum::RollAngle,
                SensorValue::Float(sim_state.roll_deg + roll_angle_noise),
            ),
            (
                SensorEnum::PitchAngle,
                SensorValue::Float(sim_state.pitch_deg + pitch_angle_noise),
            ),
            (
                SensorEnum::YawAngle,
                SensorValue::Float(sim_state.yaw_deg + yaw_angle_noise),
            ),
            (
                SensorEnum::RollRate,
                SensorValue::Float(sim_state.roll_rate_dps),
            ),
            (
                SensorEnum::PitchRate,
                SensorValue::Float(sim_state.pitch_rate_dps),
            ),
            (
                SensorEnum::YawRate,
                SensorValue::Float(sim_state.yaw_rate_dps),
            ),
            (
                SensorEnum::Latitude,
                SensorValue::Float(sim_state.latitude_deg + pitch_angle_noise),
            ),
            (
                SensorEnum::Longitude,
                SensorValue::Float(sim_state.longitude_deg + roll_angle_noise),
            ),
            (
                SensorEnum::VibrationX,
                SensorValue::Float(sim_state.vibration_x_g + vibration_noise_val_x),
            ),
            (
                SensorEnum::VibrationY,
                SensorValue::Float(sim_state.vibration_y_g + vibration_noise_val_y),
            ),
            (
                SensorEnum::VibrationZ,
                SensorValue::Float(sim_state.vibration_z_g + vibration_noise_val_z),
            ),
            (
                SensorEnum::VibrationFreq,
                SensorValue::Float(sim_state.vibration_freq_hz + vibration_freq_noise),
            ),
            // (SensorEnum::HealthStatus, SensorValue::String(sim_state.health_status.clone())),
            // (SensorEnum::MissionPhase, SensorValue::String(sim_state.mission_phase.clone())),
        ];

        for (sensor_type, value) in sensor_values {
            let jittered_timestamp = timestamp_jitter.apply(base_timestamp, &mut self.rng);
            readings.push(TelemetryReading {
                timestamp: jittered_timestamp,
                time_since_launch_ms: sim_state.time_since_launch_ms,
                sensor: sensor_type,
                value,
            });
        }

        readings
    }

    fn update_simulation_state(
        &mut self,
        state: &mut SimulationState,
        time_step_s: f64,
        idx: usize,
        total_points: usize,
    ) {
        // Todo: Too many lines here. Break into methods
        let progress: f64 = idx as f64 / total_points as f64;

        match progress {
            p if p < 0.05 => {
                // prelaunch and early lift off. (0-5%) of simulation
                let throttle_up = (p / 0.05).min(1.0);
                // debug!("Throttle up factor: {:.2} from p: {}", throttle_up, p);

                // Engine start
                state.chamber_pressure_pa = 5_000_000.0 * throttle_up; // 5 MPa max
                state.chamber_temperature_k = 3500.0 * throttle_up; // 350
                state.oxidizer_flow_rate_kgps = 250.0 * throttle_up; // 250 kg/s max
                state.fuel_flow_rate_kgps = 50.0 * throttle_up; // 50 kg/s max
                state.turbo_pump_rpm = 30_000.0 * throttle_up;
                state.thrust_n = 1_000_000.0 * throttle_up; // 1 MN max
                state.specific_impulse_s = 300.0 * throttle_up; // 300 s max
                state.nozzle_temperature_k = 3500.0 * throttle_up;

                state.acceleration_mps2 = if progress < 0.01 {
                    0.0
                } else {
                    (progress - 0.01) / 0.04 * 15.0 // ramp to 15 m/s^2
                };

                // update pos based on acceleration
                state.velocity_mps += state.acceleration_mps2 * time_step_s;
                state.altitude_m += state.velocity_mps * time_step_s;

                // Take off vibrations
                state.vibration_x_g = 0.05 * self.rng.r#gen::<f64>();
                state.vibration_y_g = 0.05 * self.rng.r#gen::<f64>();
                state.vibration_z_g = 0.1 * self.rng.r#gen::<f64>();
                state.vibration_freq_hz = 20.0 + 5.0 * self.rng.r#gen::<f64>();
            }
            p if p < 0.15 => {
                // Max-Q (5-15%)
                // Throttle down
                let max_q = 1.0 - 0.2 * ((p - 0.05) / 0.10).clamp(0.0, 1.0); // .min(1.0).max(0.0);

                state.chamber_pressure_pa = 5_000_000.0 * max_q;
                state.thrust_n = 1_000_000.0 * max_q;
                state.oxidizer_flow_rate_kgps = 250.0 * max_q;
                state.fuel_flow_rate_kgps = 50.0 * max_q;

                state.acceleration_mps2 = 15.0 * max_q;
                state.velocity_mps += state.acceleration_mps2 * time_step_s;
                state.altitude_m += state.velocity_mps * time_step_s;

                // Start gravity turn and pitch over
                state.pitch_deg = 90.0 - 15.0 * ((p - 0.05) / 0.10);
                state.pitch_rate_dps = -0.3;

                // Most vibrations here at max Q
                state.vibration_x_g = 1.0 + (1.0 - max_q) * 2.0;
                state.vibration_y_g = 1.0 + (1.0 - max_q) * 2.0;
                state.vibration_z_g = 1.5 + (1.0 - max_q) * 3.0;
                state.vibration_freq_hz = 80.0 + (1.0 - max_q) * 40.0;

                state.nozzle_temperature_k = 1500.0 + ((p - 0.05) / 0.1) * 300.0;
            }
            p if p < 0.40 => {
                // Main ascent (15-40%)
                state.chamber_pressure_pa = 5_000_000.0;
                state.thrust_n = 1_000_000.0;
                state.oxidizer_flow_rate_kgps = 250.0;
                state.fuel_flow_rate_kgps = 50.0;

                // accelerate more as fuel is consumed. 1.5x
                let acceleration_factor = 1.0 + ((p - 0.15) / 0.25) * 0.5;
                state.acceleration_mps2 = 15.0 * acceleration_factor;

                // update position
                state.velocity_mps += state.acceleration_mps2 * time_step_s;
                state.altitude_m += state.velocity_mps * time_step_s;

                // Continue gravity turn
                state.pitch_deg = 75.0 - 25.0 * ((p - 0.15) / 0.3);
                state.pitch_rate_dps = -0.1;

                // Decrease vibrations as atmosphere thins
                let vib_factor = 1.0 - ((p - 0.15) / 0.3);
                state.vibration_x_g = 0.5 * vib_factor;
                state.vibration_y_g = 0.5 * vib_factor;
                state.vibration_z_g = 0.75 * vib_factor;
                state.vibration_freq_hz = 60.0;
            }
            p if p < 0.55 => {
                // Stage separation and second stage ignition (40-55%)
                let shutdown = 1.0 - ((p - 0.45) / 0.05).min(1.0);

                state.chamber_pressure_pa = 5_000_000.0 * shutdown;
                state.thrust_n = 1_000_000.0 * shutdown;
                state.oxidizer_flow_rate_kgps = 250.0 * shutdown;
                state.fuel_flow_rate_kgps = 50.0 * shutdown;
                state.turbo_pump_rpm = 30_000.0 * shutdown;

                if p > 0.5 && p < 0.52 {
                    state.chamber_pressure_pa = 0.0;
                    state.thrust_n = 0.0;
                    state.oxidizer_flow_rate_kgps = 0.0;
                    state.fuel_flow_rate_kgps = 0.0;
                    state.turbo_pump_rpm = 0.0;
                    state.acceleration_mps2 = -9.81; // falling now
                }

                if p > 0.5 && p < 0.51 {
                    state.vibration_x_g = 3.0;
                    state.vibration_y_g = 3.0;
                    state.vibration_z_g = 5.0;
                    state.vibration_freq_hz = 100.0;
                } else {
                    state.vibration_x_g = 0.5 * shutdown;
                    state.vibration_y_g = 0.5 * shutdown;
                    state.vibration_z_g = 0.75 * shutdown;
                    state.vibration_freq_hz = 40.0 * shutdown;
                }
                // update positions
                state.acceleration_mps2 = if p < 0.5 {
                    20.0 * shutdown
                } else if p < 0.52 {
                    -9.81
                } else {
                    -9.81 + ((p - 0.52) / 0.03) * 15.0
                };
                state.velocity_mps += state.acceleration_mps2 * time_step_s;
                state.altitude_m += state.velocity_mps * time_step_s;
            }
            p if p >= 0.55 => {
                // Orbital insertion phase (55-100%)
                let stage_time = (p - 0.55) / 0.45;
                let startup = (stage_time / 20.0).min(1.0);

                state.chamber_pressure_pa = 5_000_000.0 * startup;
                state.chamber_temperature_k = 3500.0 * startup + 300.0;
                state.oxidizer_flow_rate_kgps = 250.0 * startup;
                state.fuel_flow_rate_kgps = 50.0 * startup;
                state.turbo_pump_rpm = 30_000.0 * startup;
                state.thrust_n = 2_000_000.0 * startup;
                state.specific_impulse_s = 300.0 * startup;

                state.acceleration_mps2 = 5.0 * startup;

                if stage_time > 0.9 {
                    let shutdown = 1.0 - ((stage_time - 0.9) / 0.1);
                    state.chamber_pressure_pa *= shutdown;
                    state.thrust_n *= shutdown;
                    state.oxidizer_flow_rate_kgps *= shutdown;
                    state.fuel_flow_rate_kgps *= shutdown;
                    state.turbo_pump_rpm *= shutdown;
                    state.acceleration_mps2 *= shutdown;
                }

                // update positions
                state.velocity_mps += state.acceleration_mps2 * time_step_s;
                state.altitude_m += state.velocity_mps * time_step_s;

                // Low vibrations in space vacuum
                state.vibration_x_g = 0.01 * startup;
                state.vibration_y_g = 0.01 * startup;
                state.vibration_z_g = 0.03 * startup;
                state.vibration_freq_hz = 30.0 * startup;

                state.pitch_deg = 50.0 - 40.0 * stage_time;
            }
            _ => {
                // Landing and recovery (95-100%)
                error!("Outside of p range");
                todo!();
            }
        }

        // Ensure physically realistic values
        state.chamber_pressure_pa = state.chamber_pressure_pa.max(0.0);
        state.chamber_temperature_k = state.chamber_temperature_k.max(273.0);
        state.thrust_n = state.thrust_n.max(0.0);
        state.oxidizer_flow_rate_kgps = state.oxidizer_flow_rate_kgps.max(0.0);
        state.fuel_flow_rate_kgps = state.fuel_flow_rate_kgps.max(0.0);
        state.turbo_pump_rpm = state.turbo_pump_rpm.max(1_000_000.0);

        // Update positions based on velocity and acceleration
        let distance_traveled_m = state.velocity_mps * time_step_s;
        if state.altitude_m > 100.0 && state.pitch_deg < 90.0 {
            let earth_radius_m = 6_371_000.0;
            let pitch_rad = state.pitch_deg.to_radians();
            let yaw_rad = state.yaw_deg.to_radians();
            let horizontal_distance = distance_traveled_m * pitch_rad.cos();

            // Convert to lat/lon
            let lat_change_deg = horizontal_distance * yaw_rad.sin()
                / (earth_radius_m * std::f64::consts::PI / 180.0);

            let long_change_deg = horizontal_distance * yaw_rad.sin()
                / ((earth_radius_m * std::f64::consts::PI / 180.0)
                    * state.latitude_deg.to_degrees().cos());

            state.latitude_deg += lat_change_deg;
            state.longitude_deg += long_change_deg;
        }
        // todo need else block?
    }
}

#[derive(Debug, Clone)]
struct SimulationState {
    time_since_launch_ms: u64,
    altitude_m: f64,
    velocity_mps: f64,
    acceleration_mps2: f64,
    chamber_pressure_pa: f64,
    chamber_temperature_k: f64,
    oxidizer_flow_rate_kgps: f64,
    oxidizer_pressure_pa: f64,
    oxidizer_temperature_k: f64,
    fuel_flow_rate_kgps: f64,
    fuel_pressure_pa: f64,
    fuel_temperature_k: f64,
    turbo_pump_rpm: f64,
    thrust_n: f64,
    specific_impulse_s: f64,
    nozzle_temperature_k: f64,
    roll_deg: f64,
    pitch_deg: f64,
    yaw_deg: f64,
    roll_rate_dps: f64,
    pitch_rate_dps: f64,
    yaw_rate_dps: f64,
    latitude_deg: f64,
    longitude_deg: f64,
    vibration_x_g: f64,
    vibration_y_g: f64,
    vibration_z_g: f64,
    vibration_freq_hz: f64,
    // battery_voltage_v: f64,
    // battery_current_a: f64,
    // battery_temperature_c: f64,
    // power_consumption_w: f64,
    // cpu_usage_percent: f64,
    // memory_usage_mb: f64,
    // health_status: String,
    // mission_phase: String,
    // gyroscope_dps: f64,
}

impl SimulationState {
    fn initialize() -> Self {
        SimulationState {
            time_since_launch_ms: 0,
            altitude_m: 0.0,
            velocity_mps: 0.0,
            acceleration_mps2: 0.0,
            chamber_pressure_pa: 0.0,
            chamber_temperature_k: 288.15,
            oxidizer_flow_rate_kgps: 0.0,
            oxidizer_pressure_pa: 101_325.0,
            oxidizer_temperature_k: 288.15,
            fuel_flow_rate_kgps: 0.0,
            fuel_pressure_pa: 101_325.0,
            fuel_temperature_k: 288.15,
            turbo_pump_rpm: 0.0,
            thrust_n: 0.0,
            specific_impulse_s: 0.0,
            nozzle_temperature_k: 288.15,
            roll_deg: 0.0001,
            pitch_deg: 0.0001,
            yaw_deg: 0.0001,
            roll_rate_dps: 0.0,
            pitch_rate_dps: 0.0,
            yaw_rate_dps: 0.0,
            latitude_deg: 28.5721,  // Cape Canaveral
            longitude_deg: -80.648, // Cape Canaveral
            vibration_x_g: 0.0,
            vibration_y_g: 0.0,
            vibration_z_g: 0.0,
            vibration_freq_hz: 0.0,
            // battery_voltage_v: 28.8, // Example nominal voltage
            // battery_current_a: 0.0,
            // battery_temperature_c: 25.0, // Room temperature (300k)
            // power_consumption_w: 0.0,
            // cpu_usage_percent: 5.0, // Idle CPU usage
            // memory_usage_mb: 100.0, // Example memory usage
            // health_status: "OK".into(),
            // mission_phase: "Pre-launch".into(),
            // gyroscope_dps: 0.0,
        }
    }
}
