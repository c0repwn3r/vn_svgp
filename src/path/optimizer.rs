use crate::config::Optimizer;
use crate::point::P;

mod ad_floor;
mod three_pt_average;


pub fn optimize(optimizer: &Optimizer, mut pts: Vec<P>) -> Vec<P> {
    // deduplicate
    pts.dedup();

    pts = match optimizer {
        Optimizer::ADFloor { a_floor, d_floor } => ad_floor::optimize(*a_floor, *d_floor, pts),
        Optimizer::ThreePointAverage { dt } => three_pt_average::optimize(*dt, pts),
    };

    pts
}