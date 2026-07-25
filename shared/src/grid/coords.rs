//! Lane coordinates for the triangular grid.
//!
//! `TriCoord { a, b, c }` is the three-lane scheme referenced from
//! <https://www.redblobgames.com/grids/parts/#triangle-coordinates> (credited there to Boris
//! the Brave): each axis counts steps across one of the 3 families of parallel grid lines, and
//! `a + b + c` is 1 for an upward-pointing triangle or 2 for a downward-pointing one.

use bevy::math::Vec2;
use serde::{Deserialize, Serialize};

/// A single triangular tile's grid coordinate. `a + b + c` is always 1 (upward) or 2 (downward).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TriCoord {
    pub a: i32,
    pub b: i32,
    pub c: i32,
}

impl TriCoord {
    pub fn new(a: i32, b: i32, c: i32) -> Self {
        let coord = Self { a, b, c };
        debug_assert!(
            matches!(coord.sum(), 1 | 2),
            "TriCoord sum must be 1 or 2, got {} for ({a}, {b}, {c})",
            coord.sum()
        );
        coord
    }

    pub fn sum(&self) -> i32 {
        self.a + self.b + self.c
    }

    pub fn is_upward(&self) -> bool {
        self.sum() == 1
    }

    pub fn is_downward(&self) -> bool {
        self.sum() == 2
    }

    /// World-space centroid, for a grid built from equilateral triangles of edge `edge_len`.
    ///
    /// Derived from three axis vectors 120° apart (`va` at 270°, `vb` at 30°, `vc` at 150°,
    /// each of length `edge_len / sqrt(3)`) so that stepping any single lane by one always
    /// lands on the correct edge-adjacent triangle's centroid.
    pub fn center_world(&self, edge_len: f32) -> Vec2 {
        let s = edge_len / 3f32.sqrt();
        let x = s * 3f32.sqrt() / 2.0 * (self.b - self.c) as f32;
        let y = s * (-(self.a as f32) + (self.b + self.c) as f32 / 2.0);
        Vec2::new(x, y)
    }

    /// The 3 corner points, in world space, of this triangle.
    pub fn corners_world(&self, edge_len: f32) -> [Vec2; 3] {
        let center = self.center_world(edge_len);
        let r = edge_len / 3f32.sqrt();
        let shift = if self.is_upward() { 0.0 } else { std::f32::consts::PI };
        let angles = [
            std::f32::consts::FRAC_PI_2 + shift,
            7.0 * std::f32::consts::FRAC_PI_6 + shift,
            11.0 * std::f32::consts::FRAC_PI_6 + shift,
        ];
        angles.map(|theta| center + r * Vec2::new(theta.cos(), theta.sin()))
    }

    /// Inverse of [`Self::center_world`]: which triangle contains world-space `point`.
    ///
    /// Solves the linear system for each possible orientation (`sum` of 1 or 2), rounds to the
    /// nearest integer lattice point honoring that sum, and keeps whichever candidate's centroid
    /// is closest to `point` (mirrors the standard hex cube-coordinate rounding trick).
    pub fn from_world(point: Vec2, edge_len: f32) -> TriCoord {
        let s = edge_len / 3f32.sqrt();
        let sqrt3 = 3f32.sqrt();
        // b - c, recovered from x = s * sqrt3/2 * (b - c)
        let bc_diff = 2.0 * point.x / (s * sqrt3);

        let mut best: Option<(TriCoord, f32)> = None;
        for sum in [1i32, 2i32] {
            // b + c, recovered from y = s * (-a + (b+c)/2) with a = sum - b - c
            let bc_sum = (point.y / s + sum as f32) / 1.5;
            let b = (bc_sum + bc_diff) / 2.0;
            let c = (bc_sum - bc_diff) / 2.0;
            let a = sum as f32 - b - c;

            let candidate = round_to_sum(a, b, c, sum);
            let dist = candidate.center_world(edge_len).distance_squared(point);
            if best.is_none_or(|(_, best_dist)| dist < best_dist) {
                best = Some((candidate, dist));
            }
        }
        best.expect("loop runs at least once").0
    }
}

/// Rounds real-valued `(a, b, c)` to the nearest integer triple whose sum is exactly `sum`,
/// fixing up whichever coordinate rounded with the largest error (the cube-coordinate rounding
/// trick, adapted from sum == 0 to sum == 1 or 2).
fn round_to_sum(a: f32, b: f32, c: f32, sum: i32) -> TriCoord {
    let mut ra = a.round();
    let mut rb = b.round();
    let mut rc = c.round();

    let da = (ra - a).abs();
    let db = (rb - b).abs();
    let dc = (rc - c).abs();

    let diff = sum as f32 - (ra + rb + rc);
    if da >= db && da >= dc {
        ra += diff;
    } else if db >= dc {
        rb += diff;
    } else {
        rc += diff;
    }

    TriCoord::new(ra as i32, rb as i32, rc as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orientation_matches_sum() {
        assert!(TriCoord::new(0, 0, 1).is_upward());
        assert!(TriCoord::new(0, 0, 2).is_downward());
    }

    #[test]
    fn from_world_round_trips_through_center_world() {
        let edge_len = 1.7;
        let tiles = [
            TriCoord::new(0, 0, 1),
            TriCoord::new(0, 0, 2),
            TriCoord::new(3, -1, 0),
            TriCoord::new(-2, 4, -1),
            TriCoord::new(-2, 5, -1),
            TriCoord::new(5, -3, -1),
        ];
        for tile in tiles {
            let center = tile.center_world(edge_len);
            let recovered = TriCoord::from_world(center, edge_len);
            assert_eq!(recovered, tile, "round-trip failed for {tile:?}");
        }
    }

    #[test]
    fn corners_are_equidistant_from_center() {
        let edge_len = 2.0;
        let tile = TriCoord::new(1, -1, 1);
        let center = tile.center_world(edge_len);
        let r = edge_len / 3f32.sqrt();
        for corner in tile.corners_world(edge_len) {
            assert!((corner.distance(center) - r).abs() < 1e-4);
        }
    }
}
