#![deny(clippy::unwrap_used)]
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
use clap::{arg, ArgAction, command, Command};

use crate::config::{ProgramConfig};

fn main() {
    let matches = command!()
        .arg(
            arg!(-c --config <FILE> "Configuration file")
                .required(true)
        )
        .subcommand(
            Command::new("build")
                .about("Build all aircraft JSON according to the configuration file")
        )
        .subcommand(
            Command::new("build_one")
                .arg(arg!(-a --aircraft <AID> "Single AID to build").required(true))
                .arg(arg!(-d --debug "Output CSV to stdout in addition to a json").action(ArgAction::SetTrue))
        )
        .subcommand_required(true)
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

    match matches.subcommand() {
        Some(("build", _)) => {
            let mut failures = 0;
            for (typ, cfg) in config.aircraft {
                match path::pathificate(&typ, &cfg, config.configuration.output_directory.as_path(), config.configuration.max_points) {
                    Ok(_) => (),
                    Err(e) => {
                        failures += 1;
                        eprintln!("{e:#}");
                    }
                }
            }
            if failures > 0 {
                eprintln!("{failures} aircraft could not be pathificated, please check above for details");
                exit(1);
            } else {
                println!("Pathification successful :D");
            }
        },
        Some(("build_one", m)) => {
            let aid = m.get_one::<String>("aircraft").expect("aircraft ID is required");
            let is_debug = m.get_flag("debug");

            let cfg = config.aircraft.get(aid).expect("Aircraft ID is not present in configuration");
            match path::pathificate(aid, cfg, config.configuration.output_directory.as_path(), config.configuration.max_points) {
                Ok(p) => {
                    if is_debug {
                        println!("x,y");
                        for pt in &p.points {
                            println!("{},{}", pt.x, pt.y);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("{e:#}");
                    exit(1);
                }
            }
        }
        Some((c, _)) => {
            panic!("unknown subcommand {c}")
        }
        None => panic!("subcommand is required")
    }
}





