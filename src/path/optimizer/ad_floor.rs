use crate::point::P;

pub fn optimize(a_floor: f64, d_floor: f64, pts: Vec<P>) -> Vec<P> {
    let mut prev_points = vec![];
    let mut next_points = pts;
    next_points.reverse(); // Flip, so I can pop points off the front

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
        if cur.distance(prev) < d_floor {
            continue; // drop
        }
        if cur.distance(next) < d_floor {
            continue; // drop
        }
        
        let prev_slope = (cur.y - prev.y) / (cur.x - prev.x);
        let next_slope = (next.y - cur.y) / (next.x - cur.x);

        let prev_slope_angle = prev_slope.atan();
        let next_slope_angle = next_slope.atan();

        let angle_difference = (next_slope_angle - prev_slope_angle).abs();

        if angle_difference > a_floor {
            prev_points.push(cur);
        } else {
            // drop the point
        }
    }
    
    prev_points
}