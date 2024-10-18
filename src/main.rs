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
use clap::{arg, ArgAction, ArgMatches, command, Command, value_parser};
use rayon::prelude::*;
use toml_edit::{DocumentMut, Table, TableLike};
use toml_edit::visit_mut::VisitMut;
use crate::config::{AircraftConfig, Optimizer, ProgramConfig};

#[allow(clippy::too_many_lines)]
fn main() {
    let matches = command!()
        .arg(
            arg!(-c --config <FILE> "Configuration file")
                .required(true)
        )
        .subcommand(
            Command::new("build")
                .about("Build all aircraft JSON according to the configuration file")
                .arg(arg!(-k --keepgoing "Ignore failures").action(ArgAction::SetTrue))
        )
        .subcommand(
            Command::new("minmax")
                .about("Minmax an aircraft and find the lowest a-floor and d-floor values possible")
                .arg(arg!(-a --aircraft <AID> "Single AID to minmax").required(true))
                .arg(arg!(-f --afloor <AFLOOR> "Maximum A-floor value").required(true).value_parser(value_parser!(f64)))
                .arg(arg!(-s --astep <ASTEP> "A-step").required(true).value_parser(value_parser!(f64)))
                .arg(arg!(-d --dfloor <DFLOOR> "Maximum D-floor value").required(true).value_parser(value_parser!(f64)))
                .arg(arg!(-S --dstep <DSTEP> "D-step").required(true).value_parser(value_parser!(f64)))
        )
        .subcommand(
            Command::new("minmax_all")
                .about("Minmax all aircraft and find the lowest a-floor and d-floor values possible")
                .arg(arg!(-f --afloor <AFLOOR> "Maximum A-floor value").required(true).value_parser(value_parser!(f64)))
                .arg(arg!(-s --astep <ASTEP> "A-step").required(true).value_parser(value_parser!(f64)))
                .arg(arg!(-d --dfloor <DFLOOR> "Maximum D-floor value").required(true).value_parser(value_parser!(f64)))
                .arg(arg!(-S --dstep <DSTEP> "D-step").required(true).value_parser(value_parser!(f64)))
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
    let mut config: ProgramConfig = match toml_edit::de::from_str(&config_s) {
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
        Some(("build", m)) => {
            let mut failures = 0;
            for (typ, cfg) in config.aircraft {
                let t = match path::read(&typ, &cfg) {
                    Ok(r) => r,
                    Err(e) => {
                        failures += 1;
                        eprintln!("{e:#}");
                        continue;
                    }
                };
                let pf = match path::pathificate(&typ, &cfg, config.configuration.max_points, &t) {
                    Ok(r) => r,
                    Err(e) => {
                        failures += 1;
                        eprintln!("{e:#}");
                        continue;
                    }
                };
                match path::write(&typ, config.configuration.output_directory.as_path(), &cfg, &pf) {
                    Ok(_) => (),
                    Err(e) => {
                        failures += 1;
                        eprintln!("{e:#}");
                        continue;
                    }
                }
            }
            if failures > 0 {
                eprintln!("{failures} aircraft could not be pathificated, please check above for details");
                if !m.get_flag("keepgoing") {
                    exit(1);
                }
            } else {
                println!("Pathification successful :D");
            }
        },
        Some(("build_one", m)) => {
            let aid = m.get_one::<String>("aircraft").expect("aircraft ID is required");
            let is_debug = m.get_flag("debug");

            let cfg = config.aircraft.get(aid).expect("Aircraft ID is not present in configuration");

            let t = match path::read(&aid, &cfg) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{e:#}");
                    exit(1);
                }
            };

            let pf = match path::pathificate(aid, cfg, config.configuration.max_points, &t) {
                Ok(p) => {
                    if is_debug {
                        println!("x,y");
                        for pt in &p.points {
                            println!("{},{}", pt.x, pt.y);
                        }
                    }
                    p
                },
                Err(e) => {
                    eprintln!("{e:#}");
                    exit(1);
                }
            };
            match path::write(&aid, config.configuration.output_directory.as_path(), &cfg, &pf) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{e:#}");
                    exit(1);
                }
            }
        },
        Some(("minmax", m)) => {
            let aid = m.get_one::<String>("aircraft").expect("aircraft ID is required");
            let cfc = config.clone();
            let mut cfg = config.aircraft.get_mut(aid).expect("Aircraft ID is not present in configuration");
            minmax(aid, m, cfg, &cfc);
        },
        Some(("minmax_all", m)) => {
            let cf2 = config.clone();
            config.aircraft.par_iter_mut().for_each(|u| {
                minmax(u.0, m, u.1, &cf2);
            });
            
            println!("[aircraft]");
            for (id, ac) in &config.aircraft {
                let doc = toml_edit::ser::to_document(&ac).expect("ser failed");
                let mut d2 = DocumentMut::new();
                d2.insert(id, doc.as_table().clone().into_inline_table().into());
                print!("{}", d2.to_string());
            }
        }
        Some((c, _)) => {
            panic!("unknown subcommand {c}")
        },
        None => panic!("subcommand is required")
    }
}


fn minmax(aid: &str, m: &ArgMatches, cfg: &mut AircraftConfig, config: &ProgramConfig) -> Option<(f64, f64)> {
    let afloor = m.get_one::<f64>("afloor").expect("A-floor is required");
    let astep = m.get_one::<f64>("astep").expect("A-step is required");
    let dfloor = m.get_one::<f64>("dfloor").expect("D-floor is required");
    let dstep = m.get_one::<f64>("dstep").expect("D-step is required");



    match cfg.optimizer {
        Optimizer::ADFloor { .. } => {},
        _ => return None
    }

    let mut af = 0.0;
    let mut df = 0.0;

    let max_df = *dfloor;
    let max_af = *afloor;

    let mut successful: Vec<(f64, f64)> = vec![];

    let t = match path::read(&aid, &cfg) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e:#}");
            return None;
        }
    };

    loop {
        cfg.optimizer = Optimizer::ADFloor { a_floor: af + astep, d_floor: df + dstep };

        eprintln!("a_floor={} d_floor={}", af, df);

        match path::pathificate(aid, cfg, config.configuration.max_points, &t) {
            Ok(p) => {
                eprintln!("ok p={}", p.points.len());
                successful.push((af, df));
            },
            Err(e) => {
                eprintln!("{e:#}");
            }
        }

        df += *dstep;

        if df > max_df {
            eprintln!("minmax: hit df_ceil, last parameters a_floor={} d_floor={}", af - *astep, df - *dstep);
            af += *astep;
            df = 0.0;
        }
        if af > max_af {
            eprintln!("minmax: hit af_ceil, last parameters a_floor={} d_floor={}", af - *astep, df - *dstep);
            break;
        }
    }

    eprintln!("minmax: iteration complete");
    let v = successful.iter().min_by(|u, y| {
        let a = u.0 + u.1;
        let b = y.0 + y.1;
        a.total_cmp(&b)
    })?;
    eprintln!("aid={} a_floor={} d_floor={}", aid, v.0, v.1);

    cfg.optimizer = Optimizer::ADFloor { a_floor: v.0 + astep, d_floor: v.1 + astep };

    Some(*v)
}