//! Generates the tile set for a hexagonal map made of triangles.
//!
//! `edge_tiles` is the number of triangles along one of the hexagon's 6 sides (the "edge row").
//! The hexagon is built directly as a box in [`TriCoord`]'s lane coordinates — exactly the same
//! trick used to build a hexagon out of standard hex-grid cube coordinates, just with three
//! lanes instead of the usual symmetric-around-zero three, since here `a + b + c` is 1 or 2
//! rather than 0.
//!
//! With `m = (edge_tiles - 1) / 2`, the hexagon is every tile with
//! `a ∈ [-m, m-1]`, `b ∈ [1-m, m]`, `c ∈ [2-m, m+1]`. This reproduces the row-count pattern
//! directly: row `a` has `edge_tiles + 2 * min(a - lo_a, hi_a - a)` triangles, growing by 2 per
//! row from each edge to the (doubled) middle row and back down, so the total tile count is
//! `6 * m^2`.
//!
//! Pure and seedless (the shape is a function of `edge_tiles` alone), so both the server
//! (authoritative `TileMap` generation) and the client (local map preview, before real netcode
//! exists) can call it directly.

use super::coords::TriCoord;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HexMapError {
    /// A hexagonal triangle grid needs an odd edge length so it has a well-defined center
    /// vertex; an even `edge_tiles` was given.
    MustBeOdd(i32),
    /// `edge_tiles` was too small to form a hexagon (minimum is 3: the single ring of 6
    /// triangles around one vertex).
    TooSmall(i32),
}

impl std::fmt::Display for HexMapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HexMapError::MustBeOdd(n) => {
                write!(f, "hexagon edge_tiles must be odd, got {n}")
            }
            HexMapError::TooSmall(n) => {
                write!(f, "hexagon edge_tiles must be >= 3, got {n}")
            }
        }
    }
}

impl std::error::Error for HexMapError {}

/// Every [`TriCoord`] inside the hexagonal map whose edge row has `edge_tiles` triangles.
pub fn hexagon_tiles(edge_tiles: i32) -> Result<Vec<TriCoord>, HexMapError> {
    if edge_tiles < 3 {
        return Err(HexMapError::TooSmall(edge_tiles));
    }
    if edge_tiles % 2 == 0 {
        return Err(HexMapError::MustBeOdd(edge_tiles));
    }

    let m = (edge_tiles - 1) / 2;
    let (lo_a, hi_a) = (-m, m - 1);
    let (lo_b, hi_b) = (1 - m, m);
    let (lo_c, hi_c) = (2 - m, m + 1);

    let mut tiles = Vec::with_capacity((6 * m * m) as usize);
    for a in lo_a..=hi_a {
        for b in lo_b..=hi_b {
            for sum in [1, 2] {
                let c = sum - a - b;
                if c >= lo_c && c <= hi_c {
                    tiles.push(TriCoord::new(a, b, c));
                }
            }
        }
    }
    Ok(tiles)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn expected_row_widths(edge_tiles: i32) -> Vec<i32> {
        let rows_half = edge_tiles / 2;
        let total_rows = 2 * rows_half;
        (0..total_rows)
            .map(|a| edge_tiles + 2 * a.min(total_rows - 1 - a))
            .collect()
    }

    #[test]
    fn rejects_even_edge_length() {
        assert_eq!(hexagon_tiles(4), Err(HexMapError::MustBeOdd(4)));
    }

    #[test]
    fn rejects_too_small() {
        assert_eq!(hexagon_tiles(1), Err(HexMapError::TooSmall(1)));
    }

    #[test]
    fn matches_the_worked_example() {
        // From the design doc: edge row of 15 -> middle row of 27 -> 294 triangles total.
        let tiles = hexagon_tiles(15).unwrap();
        assert_eq!(tiles.len(), 294);

        let mut by_row = std::collections::HashMap::<i32, i32>::new();
        for t in &tiles {
            *by_row.entry(t.a).or_default() += 1;
        }
        assert_eq!(*by_row.values().max().unwrap(), 27);
        assert_eq!(by_row.values().filter(|&&w| w == 27).count(), 2);
    }

    #[test]
    fn row_widths_match_formula_for_many_sizes() {
        for edge_tiles in [3, 5, 7, 9, 11, 15, 21] {
            let tiles = hexagon_tiles(edge_tiles).unwrap();
            let expected = expected_row_widths(edge_tiles);
            assert_eq!(tiles.len() as i32, expected.iter().sum::<i32>());

            let mut by_row = std::collections::HashMap::<i32, i32>::new();
            for t in &tiles {
                *by_row.entry(t.a).or_default() += 1;
            }
            let lo = *by_row.keys().min().unwrap();
            let hi = *by_row.keys().max().unwrap();
            let mut got: Vec<i32> = (lo..=hi).map(|a| by_row[&a]).collect();
            got.sort_unstable();
            let mut expected_sorted = expected.clone();
            expected_sorted.sort_unstable();
            assert_eq!(got, expected_sorted, "row widths mismatch for edge_tiles={edge_tiles}");
        }
    }

    #[test]
    fn no_duplicate_tiles() {
        let tiles = hexagon_tiles(15).unwrap();
        let unique: HashSet<_> = tiles.iter().collect();
        assert_eq!(unique.len(), tiles.len());
    }

    #[test]
    fn tile_count_formula_is_6m_squared() {
        for edge_tiles in [3, 5, 7, 9, 15, 21, 31] {
            let m = (edge_tiles - 1) / 2;
            let tiles = hexagon_tiles(edge_tiles).unwrap();
            assert_eq!(tiles.len() as i32, 6 * m * m);
        }
    }

    #[test]
    fn is_connected_via_edge_neighbors() {
        // Every non-boundary tile's edge neighbors should also be in the hexagon: a spot check
        // that the shape has no internal holes by flood-filling from one tile and comparing
        // against the full generated set.
        let tiles = hexagon_tiles(9).unwrap();
        let set: HashSet<_> = tiles.iter().copied().collect();
        let start = tiles[0];
        let mut seen = HashSet::new();
        let mut stack = vec![start];
        seen.insert(start);
        while let Some(t) = stack.pop() {
            for n in t.edge_neighbors() {
                if set.contains(&n) && seen.insert(n) {
                    stack.push(n);
                }
            }
        }
        assert_eq!(seen.len(), set.len());
    }
}
