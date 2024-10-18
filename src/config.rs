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

#[derive(Deserialize, Serialize, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct ProgramConfig {
    pub aircraft: HashMap<String, AircraftConfig>,
    pub configuration: ProgramConfigInner
}

#[derive(Deserialize, Serialize, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct AircraftConfig {
    pub f: PathBuf,
    pub attr: String,
    pub w: f64,
    pub l: f64,
    pub optimizer: Optimizer,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "t")]
pub enum Optimizer {
    #[serde(rename = "ad_floor")]
    ADFloor {
        a_floor: f64,
        d_floor: f64
    },
    #[serde(rename = "3pt_avg")]
    ThreePointAverage {
        dt: f64
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ProgramConfigInner {
    pub output_directory: PathBuf,
    pub max_points: usize
}