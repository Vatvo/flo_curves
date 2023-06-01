use crate::geo::*;
use crate::bezier::*;

use smallvec::*;

///
/// Returns true if the control polygon represented by a curve crosses the x axis
///
#[inline]
fn count_x_axis_crossings<TPoint, const N: usize>(points: &[TPoint; N]) -> usize
where
    TPoint: Coordinate + Coordinate2D,
{
    let mut num_crossings = 0;

    for idx in 0..(N-1) {
        let p1 = &points[idx];
        let p2 = &points[idx+1];

        if p1.y() < 0.0 && p2.y() > 0.0         { num_crossings += 1; }
        else if p1.y() > 0.0 && p2.y() < 0.0    { num_crossings += 1; }
    }

    let p1 = &points[0];
    let p2 = &points[N-1];

    if p1.y() < 0.0 && p2.y() > 0.0         { num_crossings += 1; }
    else if p1.y() > 0.0 && p2.y() < 0.0    { num_crossings += 1; }

    return num_crossings;
}

///
/// Returns true if the control polygon is flat enough to try to find a root for it
///
#[inline]
fn flat_enough<TPoint, const N: usize>(points: &[TPoint; N]) -> bool {
    todo!()
}

///
/// Finds an x-intercept for a bezier curve that is 'flat enough'
///
#[inline]
fn find_x_intercept<TPoint, const N: usize>(points: &[TPoint; N]) -> f64 {
    todo!()
}

///
/// Finds the points (as t-values) where a bezier curve's y coordinate is 0
///
pub fn find_roots<TPoint, const N: usize>(points: [TPoint; N]) -> SmallVec<[f64; 4]>
where
    TPoint: Coordinate + Coordinate2D,
{
    // See "A bezier curve-based root-finder", Philip J Schneider, Graphics Gems

    // List of sections waiting to be processed
    let mut sections    = vec![points];
    let mut roots       = smallvec![];

    loop {
        // Get the next section to process
        let section = if let Some(section) = sections.pop() { section } else { return roots; };

        // Find out how many times the polygon crosses the x
        let num_crossings = count_x_axis_crossings(&section);

        if num_crossings == 0 {
            // No roots if the control polygon does not cross the x-axis
            continue;
        }

        if num_crossings == 1 && flat_enough(&section) {
            // Find an x-intercept for this section
            roots.push(find_x_intercept(&section));
            continue;
        }

        // Subdivide the curve in the middle to search for more crossings
        let (left, right) = subdivideN(0.5, section);
        sections.push(right);
        sections.push(left);
    }
}
