// See "A bezier curve-based root-finder", Philip J Schneider, Graphics Gems

mod polynomial_to_bezier;
mod find_roots;
mod nearest_point_bezier_root_finder;

pub use nearest_point_bezier_root_finder::*;
