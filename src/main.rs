#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]

mod config;
mod bezier;
mod point;
mod path;

use std::fs;
use std::fs::{create_dir_all};
use std::process::exit;
use anyhow::{Context};
use clap::{arg, command};

use crate::config::{ProgramConfig};

fn main() {
    let matches = command!()
        .arg(
            arg!(-c --config <FILE> "Configuration file")
                .required(true)
        )
        .get_matches();

    let config_path: &String = if let Some(f) = matches.get_one("config") { f } else {
        eprintln!("config path is required");
        exit(1);
    };

    let config_s = match fs::read_to_string(config_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error reading configuration at {config_path}: {e}");
            exit(1);
        }
    };
    let config: ProgramConfig = match toml::from_str(&config_s) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error parsing configuration at {config_path}: {e}");
            exit(1);
        }
    };

    // create output dir
    match create_dir_all(&config.configuration.output_directory).context("Failed to create output directory") {
        Ok(()) => (),
        Err(e) => {
            eprintln!("{e:#}");
            exit(1);
        }
    }

    for (typ, cfg) in config.aircraft {
        match path::pathificate(&typ, &cfg, config.configuration.output_directory.as_path(), config.configuration.max_points) {
            Ok(()) => (),
            Err(e) => {
                eprintln!("{e:#}");
                exit(1);
            }
        }
    }
}





