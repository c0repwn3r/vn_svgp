use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::ops::{Add, Mul};
use usvg::{FillRule, Group, Node, Options, Path, Tree};
use usvg::tiny_skia_path::PathSegment;

#[derive(Serialize)]
struct AircraftPointFile {
    points: Vec<APFPoint>,
    #[serde(rename = "aircraftTypes")]
    aircraft_types: Vec<String>
}
#[derive(Serialize, PartialEq)]
struct APFPoint {
    x: f64,
    y: f64
}

#[derive(Deserialize)]
struct ConfigWrapper {
    aircraft: HashMap<String, ACConfig>,
    configuration: Config
}

#[derive(Deserialize)]
struct ACConfig {
    f: PathBuf,
    attr: String,
    w: f64,
    l: f64
}

#[derive(Deserialize)]
struct Config {
    output_directory: PathBuf
}

fn main() {
    let config = ACConfig {
        f: PathBuf::from_str("source/airbus/a320.svg").unwrap(),
        attr: "VATSIM-Radar".to_string(),
        l: 123.25,
        w: 111.833
    };

    let f = fs::read_to_string(&config.f).unwrap();
    let s = Tree::from_str(&f, &Options::default()).unwrap();

    let image_size_px = s.size();
    let image_size_px = (image_size_px.width() as f64, image_size_px.height() as f64);
    let ac_size_ft = (config.w, config.l);

    let foot_per_px = (ac_size_ft.0 / image_size_px.0, ac_size_ft.1 / image_size_px.1);

    println!("[A320][{}] scaling factor: {}w {}l", config.f.display(), foot_per_px.0, foot_per_px.1);

    let mut mpath: Option<&Path> = find_path(s.root());

    let path = mpath.expect("No path could be found :( Make sure the SVG contains 1 stroke path element");
    let stroke = path.stroke().unwrap();

    if path.data().bounds().width() == 0.0 || path.data().bounds().height() == 0.0 {
        panic!("Path is a horizontal or vertical line");
    }

    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;
    let mut first: Option<(f64, f64)> = None;

    let mut points: Vec<(f64, f64)> = vec![];

    for seg in path.data().segments() {
        match seg {
            PathSegment::MoveTo(p) => {
                if first.is_none() {
                    first = Some((p.x as f64, p.y as f64));
                }
                x = p.x as f64;
                y = p.y as f64;
            }
            PathSegment::LineTo(p) => {
                if first.is_none() {
                    first = Some((x, y));
                }
                points.push((x, y));

                x = p.x as f64;
                y = p.y as f64;
                points.push((x, y));
            }
            PathSegment::QuadTo(control, end) => {
                if first.is_none() {
                    first = Some((x, y));
                }
                points.push((x, y));
                let p0 = (x, y);
                let p1 = (control.x as f64, control.y as f64);
                let p2 = (end.x as f64, end.y as f64);

                points.push(quad(p0, p1, p2, 0.5));

                points.push(p2);

                x = p2.0;
                y = p2.1;
            }
            PathSegment::CubicTo(c1, c2, end) => {
                if first.is_none() {
                    first = Some((x, y));
                }
                points.push((x, y));
                let p0 = (x, y);
                let p1 = (c1.x as f64, c1.y as f64);
                let p2 = (c2.x as f64, c2.y as f64);
                let p3 = (end.x as f64, end.y as f64);

                points.push(cubic(p0, p1, p2, p3, 0.5));

                points.push(p3);

                x = p3.0;
                y = p3.1;
            }
            PathSegment::Close => {
                points.push((x, y));
                if let Some((x, y)) = first {
                    points.push((x, y));
                }
            }
        }
    }

    // write an output csv for now
    let f = File::create("out.csv").unwrap();
    writeln!(&f, "x,y").unwrap();

    for point in &points {
        writeln!(&f, "{},{}", point.0, point.1).unwrap();
    }

    println!("wrote {} points to out.csv", points.len());
    println!("scaling and generating real configuration");

    let mut pf = AircraftPointFile {
        points: points.iter()
            .map(|u| (u.0, -u.1)) // flip to +x +y
            .map(|u| (u.0 - image_size_px.0 / 2.0, u.1 + image_size_px.1 / 2.0)) // map to center
            .map(|u| (u.0 * foot_per_px.0, u.1 * foot_per_px.1)) // scale to rw size
            .map(|u| APFPoint { x: u.0, y: u.1 }) // objectify
            .collect::<Vec<_>>(),
        aircraft_types: vec!["A320".to_string()],
    };

    println!("optimizing: start {}", pf.points.len());
    pf.points.dedup();
    println!("optimizing: dedup {}", pf.points.len());

    let mut prev = (-f64::INFINITY, -f64::INFINITY);

    let delta_floor = 1.0;

    pf.points.retain(|u| {
        let p = (u.x, u.y);
        let d = (p.0 - prev.0, p.1 - prev.1);

        prev = (u.x, u.y);

        println!("{}, {} -> {}, {}, d {} {}", prev.0, prev.1, p.0, p.1, d.0, d.1);

        d.0.abs() >= delta_floor || d.1.abs() >= delta_floor
    });

    println!("optimizing: deltafloor {}", pf.points.len());

    let f = File::create("out2.csv").unwrap();
    writeln!(&f, "x,y").unwrap();

    for point in &pf.points {
        writeln!(&f, "{},{}", point.x, point.y).unwrap();
    }

    println!("wrote {} points to out2.csv", pf.points.len());

    fs::write("out.json", serde_json::to_string(&pf).unwrap()).unwrap();
}

fn find_path(g: &Group) -> Option<&Path> {
    for node in g.children() {
        match node {
            Node::Group(ref group) => {
                return find_path(group);
            },
            Node::Path(ref path) => {
                if path.stroke().is_some() {
                    return Some(path);
                }
            },
            Node::Image(ref image) => {
                panic!("Images will not be supported, please use a single vector path svg :(");
            },
            Node::Text(ref text) => {
                panic!("Text will not be supported, please use a single vector path svg :(");
            }
        }
    }
    None
}

#[derive(Copy, Clone, Debug)]
struct P {
    x: f64,
    y: f64
}
impl Add for P {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        P { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}
impl Mul<f64> for P {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        P { x: self.x * rhs, y: self.y * rhs }
    }
}
impl From<(f64, f64)> for P {
    fn from(value: (f64, f64)) -> Self {
        Self { x: value.0, y: value.1 }
    }
}
impl Into<(f64, f64)> for P {
    fn into(self) -> (f64, f64) {
        (self.x, self.y)
    }
}

impl Mul<P> for f64 {
    type Output = P;

    fn mul(self, rhs: P) -> Self::Output {
        P { x: rhs.x * self, y: rhs.y * self }
    }
}

fn quad(p0: (f64, f64), p1: (f64, f64), p2: (f64, f64), t: f64) -> (f64, f64) {
    if t < 0.0 || t > 1.0 {
        panic!("quad() t out of bounds");
    }
    let p0 = P::from(p0);
    let p1 = P::from(p1);
    let p2 = P::from(p2);
    ((1.0 - t).powi(2) * p0 + 2.0 * (1.0 - t) * t * p1 + t.powi(2) * p2).into()
}

fn cubic(p0: (f64, f64), p1: (f64, f64), p2: (f64, f64), p3: (f64, f64), t: f64) -> (f64, f64) {
    if t < 0.0 || t > 1.0 {
        panic!("quad() t out of bounds");
    }
    let p0 = P::from(p0);
    let p1 = P::from(p1);
    let p2 = P::from(p2);
    let p3 = P::from(p3);

    ((1.0-t).powi(3)*p0+3.0*(1.0-t).powi(2)*t*p1+3.0*(1.0-t)*t.powi(2)*p2+t.powi(3)*p3).into()
}

fn linear(p0: (f64, f64), p1: (f64, f64), t: f64) -> (f64, f64) {
    if t < 0.0 || t > 1.0 {
        panic!("quad() t out of bounds");
    }
    let p0 = P::from(p0);
    let p1 = P::from(p1);

    ((1.0-t)*p0 + t * p1).into()
}