//! Adjacency and distance for [`TriCoord`].

use super::coords::TriCoord;
use std::collections::{HashSet, VecDeque};

impl TriCoord {
    /// The 3 triangles sharing an edge with this one. Always the opposite orientation.
    pub fn edge_neighbors(&self) -> [TriCoord; 3] {
        if self.is_upward() {
            [
                TriCoord::new(self.a + 1, self.b, self.c),
                TriCoord::new(self.a, self.b + 1, self.c),
                TriCoord::new(self.a, self.b, self.c + 1),
            ]
        } else {
            [
                TriCoord::new(self.a - 1, self.b, self.c),
                TriCoord::new(self.a, self.b - 1, self.c),
                TriCoord::new(self.a, self.b, self.c - 1),
            ]
        }
    }

    /// Lane distance: the minimum number of edge crossings between two triangles.
    pub fn distance(&self, other: &TriCoord) -> i32 {
        (self.a - other.a).abs() + (self.b - other.b).abs() + (self.c - other.c).abs()
    }
}

/// All triangles within `range` edge-crossings of `center` (inclusive), including `center`.
pub fn tiles_within(center: TriCoord, range: i32) -> Vec<TriCoord> {
    let mut seen = HashSet::new();
    seen.insert(center);
    let mut frontier = VecDeque::new();
    frontier.push_back((center, 0));

    while let Some((tile, dist)) = frontier.pop_front() {
        if dist >= range {
            continue;
        }
        for neighbor in tile.edge_neighbors() {
            if seen.insert(neighbor) {
                frontier.push_back((neighbor, dist + 1));
            }
        }
    }

    seen.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_neighbors_flip_orientation() {
        let up = TriCoord::new(0, 0, 1);
        for n in up.edge_neighbors() {
            assert!(n.is_downward());
            assert_eq!(up.distance(&n), 1);
        }
    }

    #[test]
    fn edge_neighbors_are_symmetric() {
        let tile = TriCoord::new(2, -3, 2);
        for neighbor in tile.edge_neighbors() {
            assert!(
                neighbor.edge_neighbors().contains(&tile),
                "{neighbor:?} does not list {tile:?} back as a neighbor"
            );
        }
    }

    #[test]
    fn distance_matches_bfs_edge_count() {
        let start = TriCoord::new(0, 0, 1);
        for (target, expected) in [
            (TriCoord::new(0, 0, 1), 0),
            (TriCoord::new(1, 0, 1), 1),
            (TriCoord::new(1, -1, 1), 2),
        ] {
            assert_eq!(start.distance(&target), expected);
        }
    }

    #[test]
    fn tiles_within_zero_is_just_center() {
        let center = TriCoord::new(0, 0, 1);
        assert_eq!(tiles_within(center, 0), vec![center]);
    }

    #[test]
    fn tiles_within_one_is_center_plus_edge_neighbors() {
        let center = TriCoord::new(0, 0, 1);
        let mut got: Vec<_> = tiles_within(center, 1);
        got.sort();
        let mut expected: Vec<_> = std::iter::once(center)
            .chain(center.edge_neighbors())
            .collect();
        expected.sort();
        assert_eq!(got, expected);
    }
}
