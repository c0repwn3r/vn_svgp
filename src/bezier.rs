use crate::point::P;

pub fn quad(p0: P, p1: P, p2: P, t: f64) -> P {
    assert!((0.0..=1.0).contains(&t), "quad() t out of bounds");
    (1.0 - t).powi(2) * p0 + 2.0 * (1.0 - t) * t * p1 + t.powi(2) * p2
}

pub fn cubic(p0: P, p1: P, p2: P, p3: P, t: f64) -> P {
    assert!((0.0..=1.0).contains(&t), "cubic() t out of bounds");
    (1.0-t).powi(3)*p0+3.0*(1.0-t).powi(2)*t*p1+3.0*(1.0-t)*t.powi(2)*p2+t.powi(3)*p3
}