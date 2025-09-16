use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SensorType {
    Temperature,
    Pressure,
    Flow,
    Vibration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SensorEnum {
    // Flight profile
    Acceleration,
    Altitude,
    Velocity,

    // Engine
    ChamberPressure,
    ChamberTemperature,
    OxidizerPressure,
    OxidizerFlowRate,
    OxidizerTemperature,
    FuelPressure,
    FuelFlowRate,
    FuelTemperature,
    TurboPumpRpm,
    Thrust,
    SpecificImpulse,
    NozzleTemperature,

    // GNC Sensors
    RollAngle,
    PitchAngle,
    YawAngle,
    RollRate,
    PitchRate,
    YawRate,
    Latitude,
    Longitude,

    // Vibration Sensors
    VibrationX,
    VibrationY,
    VibrationZ,
    VibrationFreq,
    // Electrical System
    // BatteryVoltage,
    // BatteryCurrent,
    // BatteryTemperature,
    // PowerConsumption,

    // System Health
    // CpuUsage,
    // MemoryUsage,
    // HealthStatus,
    // MissionPhase,
    // Others
    // Gyroscope,
    // Magnetometer,
}

impl fmt::Display for SensorEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self:#?}")
    }
}

impl SensorEnum {
    // Get the unit of measurement for each sensor type
    // pub fn unit(&self) -> &'static str {
    pub fn unit(sensor_type: SensorEnum) -> &'static str {
        match sensor_type {
            SensorEnum::Acceleration => "m/s²",
            SensorEnum::Altitude => "meters",
            SensorEnum::ChamberPressure
            | SensorEnum::OxidizerPressure
            | SensorEnum::FuelPressure => "psi",
            SensorEnum::OxidizerFlowRate | SensorEnum::FuelFlowRate => "kg/s",
            SensorEnum::OxidizerTemperature
            | SensorEnum::FuelTemperature
            | SensorEnum::ChamberTemperature
            | SensorEnum::NozzleTemperature => "°C",
            SensorEnum::Velocity => "m/s",
            SensorEnum::TurboPumpRpm => "RPM",
            SensorEnum::Thrust => "N",
            SensorEnum::SpecificImpulse => "s",
            SensorEnum::RollAngle
            | SensorEnum::PitchAngle
            | SensorEnum::YawAngle
            | SensorEnum::Latitude
            | SensorEnum::Longitude => "degrees",
            SensorEnum::RollRate | SensorEnum::PitchRate | SensorEnum::YawRate => "degrees/s",
            SensorEnum::VibrationX | SensorEnum::VibrationY | SensorEnum::VibrationZ => "g",
            SensorEnum::VibrationFreq => "Hz",
            // SensorType::BatteryVoltage => "V",
            // SensorType::BatteryCurrent => "A",
            // SensorType::BatteryTemperature => "°C",
            // SensorType::PowerConsumption => "W",
            // SensorType::CpuUsage => "%",
            // SensorType::MemoryUsage => "MB",
            // SensorEnum::HealthStatus => "status",
            // SensorEnum::MissionPhase => "phase",
            // SensorType::Gyroscope => "degrees/s",
            // SensorType::Magnetometer => "µT",
        }
    }

    // Todo method to get all field_names
    // Todo could have concatenated with above method somehow?
    pub fn field_name(&self) -> &str {
        match self {
            SensorEnum::Acceleration => "acc",
            SensorEnum::Altitude => "alt",
            // SensorType::BatteryCurrent => "BatteryCurrent_a",
            // SensorType::BatteryTemperature => "BatteryTemperature_c",
            // SensorType::BatteryVoltage => "BatteryVoltage_v",
            SensorEnum::ChamberPressure => "cmb_pa",
            SensorEnum::ChamberTemperature => "cmb_k",
            // SensorType::CpuUsage => "CpuUsage_pct",
            SensorEnum::FuelFlowRate => "F_f",
            SensorEnum::FuelPressure => "F_pa",
            SensorEnum::FuelTemperature => "F_k",
            // SensorType::Gyroscope => "Gyroscope_x",
            // SensorEnum::HealthStatus => "HealthStatus",
            SensorEnum::Latitude => "Lat",
            SensorEnum::Longitude => "Lng",
            // SensorType::Magnetometer => "magnometer_t",
            // SensorType::MemoryUsage => "MemoryUsage_pct",
            // SensorEnum::MissionPhase => "MissionPhase",
            SensorEnum::NozzleTemperature => "Nz",
            SensorEnum::OxidizerFlowRate => "Ox_f",
            SensorEnum::OxidizerPressure => "ox_pa",
            SensorEnum::OxidizerTemperature => "Ox_k",
            SensorEnum::PitchAngle => "PA",
            SensorEnum::PitchRate => "PR",
            // SensorType::PowerConsumption => "PowerConsumption_pct",
            SensorEnum::RollAngle => "RA",
            SensorEnum::RollRate => "RR",
            SensorEnum::SpecificImpulse => "SI",
            SensorEnum::Thrust => "Trst",
            SensorEnum::TurboPumpRpm => "Rpm",
            SensorEnum::Velocity => "vel",
            SensorEnum::VibrationFreq => "Vb_hz",
            SensorEnum::VibrationX => "VbX",
            SensorEnum::VibrationY => "VbY",
            SensorEnum::VibrationZ => "VbZ",
            SensorEnum::YawAngle => "YA",
            SensorEnum::YawRate => "YR",
        }
    }

    pub fn field_name_full(&self) -> &str {
        match self {
            SensorEnum::Acceleration => "acceleration_mps2",
            SensorEnum::Altitude => "altitude_m",
            // SensorType::BatteryCurrent => "BatteryCurrent_a",
            // SensorType::BatteryTemperature => "BatteryTemperature_c",
            // SensorType::BatteryVoltage => "BatteryVoltage_v",
            SensorEnum::ChamberPressure => "chamber_pressure_pa",
            SensorEnum::ChamberTemperature => "chamber_temp_k",
            // SensorType::CpuUsage => "CpuUsage_pct",
            SensorEnum::FuelFlowRate => "FuelFlowRate_kgps",
            SensorEnum::FuelPressure => "FuelPressure_pa",
            SensorEnum::FuelTemperature => "FuelTemperature_k",
            // SensorType::Gyroscope => "Gyroscope_x",
            // SensorEnum::HealthStatus => "HealthStatus",
            SensorEnum::Latitude => "Latitude_deg",
            SensorEnum::Longitude => "Longitude_deg",
            // SensorType::Magnetometer => "magnometer_t",
            // SensorType::MemoryUsage => "MemoryUsage_pct",
            // SensorEnum::MissionPhase => "MissionPhase",
            SensorEnum::NozzleTemperature => "NozzleTemperature_k",
            SensorEnum::OxidizerFlowRate => "OxidizerFlowRate_kgps",
            SensorEnum::OxidizerPressure => "oxidizer_pressure_pa",
            SensorEnum::OxidizerTemperature => "OxidizerTemperature_k",
            SensorEnum::PitchAngle => "PitchAngle_deg",
            SensorEnum::PitchRate => "PitchRate_dps",
            // SensorType::PowerConsumption => "PowerConsumption_pct",
            SensorEnum::RollAngle => "RollAngle_deg",
            SensorEnum::RollRate => "RollRate_dps",
            SensorEnum::SpecificImpulse => "SpecificImpulse_s",
            SensorEnum::Thrust => "Thrust_n",
            SensorEnum::TurboPumpRpm => "TurboPumpRpm",
            SensorEnum::Velocity => "velocity_m",
            SensorEnum::VibrationFreq => "VibrationFreq_hz",
            SensorEnum::VibrationX => "VibrationX_g",
            SensorEnum::VibrationY => "VibrationY_g",
            SensorEnum::VibrationZ => "VibrationZ_g",
            SensorEnum::YawAngle => "YawAngle_deg",
            SensorEnum::YawRate => "YawRate_dps",
        }
    }

    pub fn number_of_sensors() -> usize {
        //29 // 37
        // todo get programatically
        29
        // Self::get_all_sensors_enums().len()
    }

    pub fn get_all_sensor_enums() -> Vec<SensorEnum> {
        vec![
            SensorEnum::Acceleration,
            SensorEnum::Altitude,
            // SensorType::BatteryCurrent,
            // SensorType::BatteryTemperature,
            // SensorType::BatteryVoltage,
            SensorEnum::ChamberPressure,
            SensorEnum::ChamberTemperature,
            // SensorType::CpuUsage,
            SensorEnum::FuelFlowRate,
            SensorEnum::FuelPressure,
            SensorEnum::FuelTemperature,
            // SensorType::Gyroscope,
            // SensorEnum::HealthStatus,
            SensorEnum::Latitude,
            SensorEnum::Longitude,
            // SensorType::Magnetometer,
            // SensorType::MemoryUsage,
            // SensorEnum::MissionPhase,
            SensorEnum::NozzleTemperature,
            SensorEnum::OxidizerFlowRate,
            SensorEnum::OxidizerPressure,
            SensorEnum::OxidizerTemperature,
            SensorEnum::PitchAngle,
            SensorEnum::PitchRate,
            // SensorType::PowerConsumption,
            SensorEnum::RollAngle,
            SensorEnum::RollRate,
            SensorEnum::SpecificImpulse,
            SensorEnum::Thrust,
            SensorEnum::TurboPumpRpm,
            SensorEnum::Velocity,
            SensorEnum::VibrationFreq,
            SensorEnum::VibrationX,
            SensorEnum::VibrationY,
            SensorEnum::VibrationZ,
            SensorEnum::YawAngle,
            SensorEnum::YawRate,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SensorValue {
    Float(f64),
    // Int(i64),
    // UnsignedInt(u64),
    String(String),
    // State(u8),
    // Status(u32),
}
