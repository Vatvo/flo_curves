use crate::bezier::vectorize::*;

use smallvec::*;

use std::ops::{Range};

///
/// Provides an implementation of `SampledContour` derived from a ray-casting function that
/// provides the intercepts along any y position
///
pub struct RayCastContour<TFn>
where
    TFn: Fn(f64) -> SmallVec<[Range<f64>; 4]>
{
    /// Function that maps a y position to the intercepts on the x axis
    intercept_fn: TFn,

    /// Size of the contour to return from this function
    size: ContourSize,

    /// Scale factor used to convert y positions into positions to pass to the intercept function
    scale_factor: f64,
}

impl<TFn> RayCastContour<TFn>
where
    TFn: Fn(f64) -> SmallVec<[Range<f64>; 4]>,
{
    ///
    /// Creates a new ray-cast contour
    ///
    /// The `intercept_fn` defines where the edges of the contour are by finding the ranges that are inside the contour at a 
    /// given y position. The size indicates the range of x and y positions that can be generated by the contour.
    ///
    /// The function must return the intercepts in ascending x order.
    ///
    #[inline]
    pub fn new(intercept_fn: TFn, size: ContourSize) -> Self {
        RayCastContour { 
            intercept_fn:   intercept_fn, 
            size:           size, 
            scale_factor:   1.0
        }
    }

    ///
    /// Sets the scale factor for the contour
    ///
    #[inline]
    pub fn with_scale(self, scale_factor: f64) -> Self {
        RayCastContour {
            intercept_fn:   self.intercept_fn,
            size:           self.size,
            scale_factor:   scale_factor
        }
    }
}

impl<TFn> Clone for RayCastContour<TFn>
where
    TFn: Clone + Fn(f64) -> SmallVec<[Range<f64>; 4]>,
{
    #[inline]
    fn clone(&self) -> Self {
        RayCastContour {
            intercept_fn:   self.intercept_fn.clone(),
            size:           self.size,
            scale_factor:   self.scale_factor,
        }
    }
}

impl<TFn> SampledContour for RayCastContour<TFn>
where
    TFn: Fn(f64) -> SmallVec<[Range<f64>; 4]>,
{
    ///
    /// The size of this contour
    ///
    #[inline]
    fn contour_size(&self) -> ContourSize { 
        self.size
    }

    ///
    /// Given a y coordinate returns ranges indicating the filled pixels on that line
    ///
    /// The ranges must be provided in ascending order, and must also not overlap.
    ///
    fn intercepts_on_line(&self, y: f64) -> SmallVec<[Range<f64>; 4]> {
        raycast_intercepts_on_line(&self.intercept_fn, y, self.scale_factor, self.size)
    }
}

///
/// Implementation of the `intercepts_on_line` trait function from `SampledContour`, implemented in terms of a function that
/// returns where the intercepts are
///
#[inline]
pub (crate) fn raycast_intercepts_on_line<TFn>(intercept_fn: &TFn, y: f64, scale_factor: f64, size: ContourSize) -> SmallVec<[Range<f64>; 4]> 
where
    TFn: Fn(f64) -> SmallVec<[Range<f64>; 4]>,
{
    // Convert the y position to a coordinate
    let y       = y as f64;
    let y       = y * scale_factor;
    let width   = size.width() as f64;

    // Find the intercepts on this line
    let intercepts = (intercept_fn)(y);

    // Process them to create the final result: remove intercepts outside of the width of the cell, clip the remaining intercepts, round to usizes and then remove any 0-width intercepts
    intercepts.into_iter()
        .filter(|intercept| intercept.end >= 0.0 && intercept.start < width)
        .map(|intercept| {
            let start   = if intercept.start < 0.0 { 0.0 } else { intercept.start };
            let end     = if intercept.end >= width { width } else { intercept.end };
            start..end
        })
        .filter(|intercept| intercept.start < intercept.end)
        .collect()
}

/// Ray cast contour with a dynamic intercept function
pub type DynRayCastContour = RayCastContour<Box<dyn Fn(f64) -> SmallVec<[Range<f64>; 4]>>>;
