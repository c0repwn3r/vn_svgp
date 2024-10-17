use std::fs;
use anyhow::{bail, Context};
use usvg::{Options, Tree};
use usvg::tiny_skia_path::PathSegment;
use crate::bezier;
use crate::config::{AircraftConfig, AircraftPointFile};
use crate::path::points::points_on_path;
use crate::point::P;

pub mod utils;
mod points;

pub fn pathificate(ac_typ: &str, config: &AircraftConfig, out: &std::path::Path, max_points: usize) -> anyhow::Result<()> {
    let svg_str = fs::read_to_string(&config.f)
        .with_context(|| format!("[{}:{}] failed to read svg from {}", ac_typ, &config.f.display(), &config.f.display()))?;
    let svg_tree = Tree::from_str(&svg_str, &Options::default())
        .with_context(|| format!("[{}:{}] failed to parse svg", ac_typ, &config.f.display()))?;

    let image_size_px = svg_tree.size();
    let image_size_px = (f64::from(image_size_px.width()), f64::from(image_size_px.height()));
    let ac_size_ft = (config.w, config.l);

    let foot_per_px = (ac_size_ft.0 / image_size_px.0, ac_size_ft.1 / image_size_px.1);

    let points = points_on_path(
        utils::find_path(svg_tree.root())
            .with_context(|| format!("[{}:{}] No path element could be found :( Make sure the SVG contains at least 1 path element with a solid stroke", ac_typ, &config.f.display()))?,
        ac_typ,
        config
    )
        .with_context(|| format!("[{}:{}] Failed to calculate points on path :(", ac_typ, &config.f.display()))?;
    
    let points = points.iter()
        .map(|u| P::from((u.x - image_size_px.0 / 2.0, u.y + image_size_px.1 / 2.0))) // map to center
        .map(|u| P::from((u.x * foot_per_px.0, u.y * foot_per_px.1))) // map to worldspace
        .collect::<Vec<_>>(); // turn back into a Vec<P>
    
    let mut pf = AircraftPointFile {
        points,
        aircraft_types: vec![ac_typ.to_string()],
        attribution: config.attr.clone()
    };

    // Optimizing: Remove duplicate points
    pf.points.dedup();
    
    let mut prev_points = vec![];
    let mut next_points = pf.points;
    next_points.reverse(); // Flip, so I can pop points off the front
    
    let slope_floor = 0.7;
    
    while let Some(cur) = next_points.pop() {
        if prev_points.is_empty() {
            prev_points.push(cur);
            continue;
        }
        
        if next_points.is_empty() {
            prev_points.push(cur);
            break; // always include the last point
        }
        
        let prev = &prev_points[prev_points.len()-1];
        let next = &next_points[next_points.len()-1];
        
        // if we are very very close to the previous or next point, we can skip this one
        if cur.distance(prev) < config.d_floor {
            continue; // drop
        }
        if cur.distance(next) < config.d_floor {
            continue; // drop
        }
        
        
        
        let prev_slope = (cur.y - prev.y) / (cur.x - prev.x);
        let next_slope = (next.y - cur.y) / (next.x - cur.x);
        
        let prev_slope_angle = prev_slope.atan();
        let next_slope_angle = next_slope.atan();
        
        let angle_difference = (next_slope_angle - prev_slope_angle).abs();
        
        if angle_difference > config.a_floor {
            prev_points.push(cur);
        } else {
            // drop the point
        }
    }
    
    pf.points = prev_points;

    println!("x,y");
    for point in &pf.points {
        println!("{},{}", point.x, point.y);
    }
    
    if pf.points.len() > max_points {
        bail!("[{}:{}] Too many points! {} points after optimization is above limit of {}, try increasing the a-floor or simplifying your SVG", ac_typ, &config.f.display(), pf.points.len(), max_points);
    }

    let p = out.join(format!("{ac_typ}.json"));

    fs::write(
        &p,
        serde_json::to_string(&pf)
            .with_context(|| format!("[{}:{}] failed to serialize path spec", ac_typ, &config.f.display()))?
    )
        .with_context(|| format!("[{}:{}] failed to write path spec to {}", ac_typ, &config.f.display(), &p.display()))?;

    //println!("[{} {}] pathificated -> {} points", ac_typ, &config.f.display(), pf.points.len());

    Ok(())
}