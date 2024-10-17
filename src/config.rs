use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::point::P;

#[derive(Serialize)]
#[allow(clippy::module_name_repetitions)]
pub struct AircraftPointFile {
    pub points: Vec<P>,
    #[serde(rename = "aircraftTypes")]
    pub aircraft_types: Vec<String>,
    pub attribution: String
}

#[derive(Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct ProgramConfig {
    pub aircraft: HashMap<String, AircraftConfig>,
    pub configuration: ProgramConfigInner
}

#[derive(Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct AircraftConfig {
    pub f: PathBuf,
    pub attr: String,
    pub w: f64,
    pub l: f64,
    pub a_floor: f64,
    pub d_floor: f64
}

#[derive(Deserialize)]
pub struct ProgramConfigInner {
    pub output_directory: PathBuf,
    pub max_points: usize
}