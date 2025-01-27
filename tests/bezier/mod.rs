#![allow(clippy::all)]  // Tests are lower priority to fix

use flo_curves::*;
use flo_curves::bezier;

mod path;
mod algorithms;

mod basis;
mod section;
mod subdivide;
mod derivative;
mod tangent;
mod normal;
mod bounds;
mod deform;
mod search;
mod solve;
mod offset;
mod overlaps;
mod intersection;
mod characteristics;
mod self_intersection;
mod curve_intersection_clip;
mod length;
mod walk;
mod distort;
mod nearest_point_tests;

pub fn approx_equal(a: f64, b: f64) -> bool {
    f64::floor(f64::abs(a-b)*10000.0) == 0.0
}

#[test]
fn read_curve_control_points() {
    let curve = bezier::Curve::from_points(Coord2(1.0, 1.0), (Coord2(3.0, 3.0), Coord2(4.0, 4.0)), Coord2(2.0, 2.0));

    assert!(curve.start_point() == Coord2(1.0, 1.0));
    assert!(curve.end_point() == Coord2(2.0, 2.0));
    assert!(curve.control_points() == (Coord2(3.0, 3.0), Coord2(4.0, 4.0)));
}

#[test]
fn read_curve_points() {
    let curve = bezier::Curve::from_points(Coord2(1.0, 1.0), (Coord2(3.0, 3.0), Coord2(4.0, 4.0)), Coord2(2.0, 2.0));

    for x in 0..100 {
        let t = (x as f64)/100.0;

        let point           = curve.point_at_pos(t);
        let another_point   = bezier::de_casteljau4(t, Coord2(1.0, 1.0), Coord2(3.0, 3.0), Coord2(4.0, 4.0), Coord2(2.0, 2.0));

        assert!(point.distance_to(&another_point) < 0.001);
    }
}
