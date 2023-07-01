use flo_curves::arc::*;
use flo_curves::bezier::*;
use flo_curves::bezier::path::*;
use flo_curves::bezier::rasterize::*;
use flo_curves::bezier::vectorize::*;

#[test]
fn corners_are_outside() {
    let radius          = 300.0;
    let center          = Coord2(500.0, 500.0);
    let circle_path     = Circle::new(center, radius).to_path::<SimpleBezierPath>();

    let circle_field    = PathDistanceField::from_path(vec![circle_path], ContourSize(1000, 1000));

    assert!(circle_field.distance_at_point(ContourPosition(0, 0)) > 0.0, "Distance at 0,0 is {:?}", circle_field.distance_at_point(ContourPosition(0, 0)));
    assert!(circle_field.distance_at_point(ContourPosition(999, 0)) > 0.0, "Distance at 999,0 is {:?}", circle_field.distance_at_point(ContourPosition(999, 0)));
    assert!(circle_field.distance_at_point(ContourPosition(0, 999)) > 0.0, "Distance at 0,999 is {:?}", circle_field.distance_at_point(ContourPosition(0, 999)));
    assert!(circle_field.distance_at_point(ContourPosition(999, 999)) > 0.0, "Distance at 999,999 is {:?}", circle_field.distance_at_point(ContourPosition(999, 999)));
}

#[test]
fn center_is_inside() {
    let radius          = 300.0;
    let center          = Coord2(500.0, 500.0);
    let circle_path     = Circle::new(center, radius).to_path::<SimpleBezierPath>();

    let circle_field    = PathDistanceField::from_path(vec![circle_path], ContourSize(1000, 1000));

    println!("{:?}", circle_field.distance_at_point(ContourPosition(501, 501)));

    assert!(circle_field.distance_at_point(ContourPosition(501, 501)) < 0.0, "Distance at center is {:?}", circle_field.distance_at_point(ContourPosition(501, 501)));
    assert!(circle_field.distance_at_point(ContourPosition(500, 500)) < 0.0, "Distance near center is {:?}", circle_field.distance_at_point(ContourPosition(500, 500)));
    assert!(circle_field.distance_at_point(ContourPosition(499, 499)) < 0.0, "Distance near center is {:?}", circle_field.distance_at_point(ContourPosition(499, 499)));
}

#[test]
fn outside_point_distances() {
    let radius          = 300.0;
    let center          = Coord2(500.0, 500.0);
    let circle_path     = Circle::new(center, radius).to_path::<SimpleBezierPath>();

    let circle_field    = PathDistanceField::from_path(vec![circle_path], ContourSize(1000, 1000));

    for y in 0..1000 {
        for x in 0..1000 {
            let to_center = Coord2(x as _, y as _).distance_to(&center);

            if to_center <= radius {
                continue;
            }

            let field_distance = circle_field.distance_at_point(ContourPosition(x, y));

            assert!(field_distance >= 0.0, "Distance at {}, {} is {} (this point is not inside the circle)", x, y, field_distance);
            assert!((field_distance-(to_center-300.0)).abs() < 2.0, "Distance at {}, {} is {} ({} to center)", x, y, field_distance, to_center);
        }
    }
}

#[test]
fn inside_point_distances() {
    let radius          = 300.0;
    let center          = Coord2(500.0, 500.0);
    let circle_path     = Circle::new(center, radius).to_path::<SimpleBezierPath>();

    let circle_field    = PathDistanceField::from_path(vec![circle_path], ContourSize(1000, 1000));

    for y in 0..1000 {
        for x in 0..1000 {
            let to_center = Coord2(x as _, y as _).distance_to(&center);

            if to_center >= radius {
                continue;
            }

            let field_distance = circle_field.distance_at_point(ContourPosition(x, y));

            assert!(field_distance <= 0.0, "Distance at {}, {} is {} (this point is not outside the circle)", x, y, field_distance);
            assert!((field_distance-(to_center-300.0)).abs() < 2.0, "Distance at {}, {} is {} ({} to center)", x, y, field_distance, to_center);
        }
    }
}

#[test]
fn trace_circle_without_distance_field() {
    // This is the equivalent of trace_circle except we don't load it into a distance field first
    // If this test fails, then the other test will likely fail due to a problem with tracing the points rather than the distance field
    let radius          = 300.0;
    let center          = Coord2(500.0, 500.0);
    let circle_path     = Circle::new(center, radius).to_path::<SimpleBezierPath>();

    let circle_points   = circle_path.to_curves::<Curve<_>>()
        .into_iter()
        .flat_map(|curve| {
            walk_curve_evenly_map(curve, 1.0, 0.1, |section| section.point_at_pos(1.0))
        })
        .collect::<Vec<_>>();
    let traced_circle   = fit_curve::<Curve<_>>(&circle_points, 0.1).unwrap();

    debug_assert!(traced_circle.len() < 20, "Result has {} curves", traced_circle.len());

    let mut num_points = 0;
    for curve in traced_circle {
        for t in 0..100 {
            num_points += 1;

            let t           = (t as f64) / 100.0;
            let point       = curve.point_at_pos(t);

            let distance    = point.distance_to(&Coord2(500.0, 500.0));

            debug_assert!((distance - radius) < 1.0, "Point #{} at distance {:?}", num_points, distance);
        }
    }
}

#[test]
fn trace_circle() {
    let radius          = 300.0;
    let center          = Coord2(500.0, 500.0);
    let circle_path     = Circle::new(center, radius).to_path::<SimpleBezierPath>();

    let circle_field    = PathDistanceField::from_path(vec![circle_path], ContourSize(1000, 1000));
    let traced_circle   = trace_paths_from_distance_field::<SimpleBezierPath>(&circle_field, 0.1);

    debug_assert!(traced_circle.len() == 1);
    debug_assert!(traced_circle[0].to_curves::<Curve<_>>().len() < 20, "Result has {} curves", traced_circle[0].to_curves::<Curve<_>>().len());

    let mut num_points = 0;
    for curve in traced_circle[0].to_curves::<Curve<_>>() {
        for t in 0..100 {
            num_points += 1;

            let t           = (t as f64) / 100.0;
            let point       = curve.point_at_pos(t);

            let distance    = point.distance_to(&Coord2(500.0, 500.0));

            debug_assert!((distance - radius) < 1.0, "Point #{} at distance {:?}", num_points, distance);
        }
    }
}
