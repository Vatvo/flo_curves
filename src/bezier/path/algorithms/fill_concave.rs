use super::fill_convex::*;
use super::fill_settings::*;

use crate::geo::*;
use crate::line::*;
use crate::bezier::*;
use crate::bezier::path::*;

use std::f64;

///
/// Item returned from a ray cast intersection by the concave raycasting function (where we can hit an existing edge)
///
#[derive(Clone, Debug)]
enum ConcaveItem<Item> {
    /// Edge returned by the main raycasting function
    Edge(Item),

    /// Intersection with an edge detected in an earlier raycasting operation
    SelfIntersection(usize)
}

impl<Item> Into<Option<Item>> for ConcaveItem<Item> {
    fn into(self) -> Option<Item> {
        match self {
            ConcaveItem::Edge(item)             => Some(item),
            ConcaveItem::SelfIntersection(_)    => None
        }
    }
}

///
/// Represents a long edge that we want to raycast from
///
struct LongEdge<Coord> {
    start:          Coord,
    end:            Coord,
    edge_index:     (usize, usize),
    ray_collided:   bool
}

///
/// Retrieves the 'long' edges from a set of edges returned by a raycast tracing operation
///
fn find_long_edges<Coord, Item>(edges: &[RayCollision<Coord, Item>], edge_min_len_squared: f64) -> Vec<LongEdge<Coord>>
where Coord: Coordinate+Coordinate2D {
    // Find the edges where we need to cast extra rays
    let mut long_edges      = vec![];
    for edge_num in 0..edges.len() {
        // Get the length of this edge
        let last_edge               = if edge_num == 0 {
            edges.len() - 1
        } else {
            edge_num-1
        };

        let edge_offset             = edges[last_edge].position - edges[edge_num].position;
        let edge_distance_squared   = edge_offset.dot(&edge_offset);

        // Add to the list of long edges if it's long enough to need further ray-casting
        if edge_distance_squared >= edge_min_len_squared {
            long_edges.push(LongEdge { 
                start:          edges[last_edge].position.clone(),
                end:            edges[edge_num].position.clone(),
                edge_index:     (last_edge, edge_num),
                ray_collided:   false
            })
        }
    }

    long_edges
}

///
/// Traces the outline of a complex area using ray-casting
///
/// While the convex version of this function can only trace the outline of a region as it can be reached by a single ray, this
/// concave version can trace outlines with edges that are not in 'line of sight' from the origin point. The extra work required
/// for this can be quite time-consuming, so an efficient ray-casting function is vital if the path is particularly complex.
///
/// The current version of the algorithm works by taking the result from a convex ray-cast and finding large gaps where no points
/// were detected, and filling them in by ray-casting from there. There are cases where the resulting path can overlap itself: after
/// fitting the curve, use `remove_interior_points` to generate a non-overlapping path.
///
/// Collisions generated internally will have 'None' set for the `what` field of the ray collision (this is why the field is made
/// optional by this call)
///
pub fn trace_outline_concave<Coord, Item, RayList, RayFn>(center: Coord, options: &FillSettings, cast_ray: RayFn) -> Vec<RayCollision<Coord, Option<Item>>> 
where   Coord:      Coordinate+Coordinate2D,
        RayList:    IntoIterator<Item=RayCollision<Coord, Item>>,
        RayFn:      Fn(Coord, Coord) -> RayList {
    // Modify the raycasting function to return concave items (so we can distinguish between edges we introduced and ones matched by the original raycasting algorithm)
    // TODO: this just ensures we return optional items
    let cast_ray                    = &cast_ray;
    let cast_ray                    = &|from, to| {
        cast_ray(from, to).into_iter().map(|collision| {
            RayCollision {
                position:   collision.position,
                what:       ConcaveItem::Edge(collision.what)
            }
        })
    };

    // The edge min length is the length of edge we need to see before we'll 'look around' a corner
    let edge_min_len                = options.step * 4.0;
    let edge_min_len_squared        = edge_min_len * edge_min_len;

    // Distnace to move past a self-intersection (so we fully close the path)
    let self_intersection_distance  = 0.5;

    // Perform the initial convex ray-casting
    let mut edges = trace_outline_convex(center, options, cast_ray);

    // Stop if we found no collisions
    if edges.len() < 2 {
        return vec![];
    }

    // Find the edges where we need to cast extra rays
    let mut long_edges      = find_long_edges(&edges, edge_min_len_squared);

    // TODO: cast rays from each of the 'long' edges and update the edge list
    let mut long_edge_index = 0;
    while long_edge_index < long_edges.len() {
        // Fetch the next edge to cast from
        let next_edge = &long_edges[long_edge_index];

        // Skip edges where we've already self-intersected
        if !next_edge.ray_collided {
            // Pick the center point
            let center_point = (next_edge.start + next_edge.end) * 0.5;
            let offset      = next_edge.start - next_edge.end;

            // Find the angle of the next edge
            let line_angle  = offset.x().atan2(offset.y());

            // Generate a version of the raycasting function that inspects the existing list of long edges
            let cast_ray_to_edges   = |from: Coord, to: Coord| {
                // Generate the edge collisions from the main raycasting function
                let edge_collisions     = cast_ray(from.clone(), to.clone());
                let ray_line            = (from.clone(), to.clone());

                // Generate the collisions with the 'long edges' where we'll be considering casting more rays later on
                let extra_collisions    = long_edges.iter()
                    .enumerate()
                    .filter(|(edge_index, _edge)| *edge_index != long_edge_index)
                    .filter_map(move |(edge_index, edge)| {
                        // Create lines from the ray and the lines
                        let edge_line   = (edge.start.clone(), edge.end.clone());

                        // Detect where they intersect
                        if let Some(intersection_point) = line_intersects_ray(&edge_line, &ray_line) {
                            // Move the intersection point slightly inside the shape along the direction of the ray (so we can add the final result up properly)
                            let length              = to.distance_to(&from);
                            let direction           = (to-from) * (1.0/length);
                            let intersection_point  = intersection_point + (direction * self_intersection_distance);
                            let pos_on_line         = edge_line.pos_for_point(&intersection_point);

                            if pos_on_line >= 0.0 && pos_on_line <= 1.0 {
                                // Generate a colision at this point
                                Some(RayCollision {
                                    position:   intersection_point,
                                    what:       ConcaveItem::SelfIntersection(edge_index)
                                })
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    });

                // Combine the two sets to generate the final set of collisions
                edge_collisions.into_iter().chain(extra_collisions)
            };

            // Perform raycasting over a 180 degree angle to get the next set of edges
            let new_edges   = trace_outline_convex_partial(center_point, options, line_angle..(line_angle+f64::consts::PI), cast_ray_to_edges);

            if new_edges.len() > 2 {
                // We ignore the first and last point as they will be the points along the existing edge (ie, will be the start and end points we already know)
                let next_edge_index = next_edge.edge_index.1;

                // Invalidate any edge we've had an intersection with (we'll end up with a 0-width gap we'll try to fill if we process these)
                for new_edge in new_edges.iter() {
                    if let ConcaveItem::SelfIntersection(edge_index) = new_edge.what {
                        long_edges[edge_index].ray_collided = true;
                    }
                }

                // Find new long edges in the new edges
                let mut new_long_edges  = find_long_edges(&new_edges[1..(new_edges.len()-1)], edge_min_len_squared);

                // Don't count the edge ending at point 0 (that's the edge we just came from)
                new_long_edges.retain(|edge| edge.edge_index.1 != 0);

                // Insert the new edges into the existing edge list (except the first and last which will be duplicates)
                let edge_index      = next_edge_index;
                let num_new_edges   = new_edges.len()-2;
                edges.splice(edge_index..edge_index, new_edges.into_iter().skip(1).take(num_new_edges));

                // Update the remaining long edge indexes
                for update_idx in long_edge_index..long_edges.len() {
                    if long_edges[update_idx].edge_index.0 >= edge_index {
                        long_edges[update_idx].edge_index.0 += num_new_edges;
                    }

                    if long_edges[update_idx].edge_index.1 >= edge_index {
                        long_edges[update_idx].edge_index.1 += num_new_edges;
                    }
                }

                // Add the new long edges to the list
                for edge in new_long_edges.iter_mut() {
                    edge.edge_index.0 += edge_index;
                    edge.edge_index.1 += edge_index;
                }

                long_edges.splice((long_edge_index+1)..(long_edge_index+1), new_long_edges);
            }
        }

        // Check the next edge
        long_edge_index += 1;
    }

    // The edges we retrieved are the result
    edges.into_iter()
        .map(|collision| RayCollision { 
            position:   collision.position,
            what:       collision.what.into()
        })
        .collect()
}

///
/// Creates a Bezier path by flood-filling a convex area whose bounds can be determined by ray-casting.
/// 
/// This won't fill areas that cannot be directly reached by a straight line from the center point. If the
/// area is not entirely closed (from the point of view of the ray-casting function), then a line will be
/// generated between the gaps.
///
pub fn flood_fill_concave<Path, Coord, Item, RayList, RayFn>(center: Coord, options: &FillSettings, cast_ray: RayFn) -> Option<Vec<Path>>
where   Path:       BezierPathFactory<Point=Coord>,
        Coord:      Coordinate+Coordinate2D,
        RayList:    IntoIterator<Item=RayCollision<Coord, Item>>,
        RayFn:      Fn(Coord, Coord) -> RayList {
    // Trace where the ray casting algorithm indicates collisions with the specified center
    let collisions = trace_outline_concave(center, options, cast_ray);

    // Build a path using the LMS algorithm
    let curves = fit_curve::<Curve<Coord>>(&collisions.iter().map(|collision| collision.position.clone()).collect::<Vec<_>>(), options.fit_error);

    if let Some(curves) = curves {
        if curves.len() > 0 {
            // Convert the curves into a path
            let initial_point   = curves[0].start_point();
            let overlapped_path  = Path::from_points(initial_point, curves.into_iter().map(|curve| {
                let (cp1, cp2)  = curve.control_points();
                let end_point   = curve.end_point();
                (cp1, cp2, end_point)
            }));
            
            // Remove any interior points that the path might have (this happens when the fill path overlaps itself)
            Some(path_remove_interior_points(&vec![overlapped_path], 0.01))
        } else {
            // No curves in the path
            None
        } 
    } else {
        // Failed to fit a curve to these points
        None
    }
}