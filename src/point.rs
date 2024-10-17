use std::fmt::{Display, Formatter};
use std::ops::{Add, Mul};
use serde::{Deserialize, Serialize};
use usvg::tiny_skia_path::Point;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct P {
    pub x: f64,
    pub y: f64
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
impl From<P> for (f64, f64) {
    fn from(val: P) -> (f64, f64) {
        (val.x, val.y)
    }
}

impl Mul<P> for f64 {
    type Output = P;

    fn mul(self, rhs: P) -> Self::Output {
        P { x: rhs.x * self, y: rhs.y * self }
    }
}
impl From<Point> for P {
    fn from(value: Point) -> Self {
        Self { x: f64::from(value.x), y: f64::from(value.y) }
    }
}
impl P {
    pub fn distance(&self, other: &P) -> f64 {
        ((other.x - self.x).powi(2) + (other.y - self.y).powi(2)).sqrt()
    }
}

impl Display for P {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}