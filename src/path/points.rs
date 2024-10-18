use anyhow::{bail, Context};
use usvg::Path;
use usvg::tiny_skia_path::PathSegment;
use crate::bezier;
use crate::config::AircraftConfig;
use crate::point::P;

#[allow(clippy::module_name_repetitions)]
pub fn points_on_path(path: &Path, ac_typ: &str, config: &AircraftConfig) -> anyhow::Result<Vec<P>> {
    path.stroke().with_context(|| format!("[{}:{}] No path element could be found :( Make sure the SVG contains at least 1 path element with a solid stroke", ac_typ, &config.f.display()))?;

    if path.data().bounds().width() == 0.0 || path.data().bounds().height() == 0.0 {
        bail!("Path found is a horizontal or vertical line");
    }

    let mut pos: P = P::from((0.0, 0.0));
    let mut first: Option<P> = None;

    let mut points: Vec<P> = vec![];

    for seg in path.data().segments() {
        match seg {
            PathSegment::MoveTo(p) => {
                if first.is_none() {
                    first = Some(p.into());
                }
                pos = p.into();
            }
            PathSegment::LineTo(p) => {
                if first.is_none() {
                    first = Some(pos);
                }
                points.push(p.into());

                pos = p.into();

                points.push(p.into());
            }
            PathSegment::QuadTo(control, end) => {
                if first.is_none() {
                    first = Some(pos);
                }
                points.push(pos);
                let p0 = pos;
                let p1 = P::from(control);
                let p2 = P::from(end);

                points.push(bezier::quad(p0, p1, p2, 0.5));

                points.push(p2);

                pos = p2;
            }
            PathSegment::CubicTo(c1, c2, end) => {
                if first.is_none() {
                    first = Some(pos);
                }
                points.push(pos);
                let p0 = pos;
                let p1 = P::from(c1);
                let p2 = P::from(c2);
                let p3 = P::from(end);

                points.push(bezier::cubic(p0, p1, p2, p3, 0.5));

                points.push(p3);

                pos = p3;
            }
            PathSegment::Close => {
                points.push(pos);
                if let Some(pos) = first {
                    points.push(pos);
                }
            }
        }
    }
    
    Ok(points.iter()
        .map(|u| P::from((u.x, -u.y))) // flip to +x +y
        .collect::<Vec<_>>())
}