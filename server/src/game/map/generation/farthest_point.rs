//! Greedy farthest-point sampling: picks a random first point from `candidates`, then repeatedly
//! adds whichever remaining candidate maximizes its minimum distance to already-chosen points.
//! This is the standard technique for a well-spread ("blue noise") subset of an irregular,
//! possibly non-convex domain — i.e. "as equidistant as possible" — and is shared by both Biome
//! Town placement (`towns.rs`, over every land tile) and starting-town selection
//! (`starting_towers.rs`, over the already-placed towns).

use rand::Rng;
use triactory_shared::grid::TriCoord;

/// Returns `min(count, candidates.len())` points from `candidates`, spread as evenly as a greedy
/// algorithm can manage. The first point is a seeded random choice; ties for "farthest" are
/// broken by another seeded random choice, so repeated calls with the same seed and inputs are
/// deterministic but different seeds (or candidate sets) visibly vary the result.
pub(crate) fn farthest_point_sample(
    candidates: &[TriCoord],
    count: usize,
    rng: &mut impl Rng,
) -> Vec<TriCoord> {
    let target = count.min(candidates.len());
    if target == 0 {
        return Vec::new();
    }

    let mut chosen: Vec<TriCoord> = Vec::with_capacity(target);
    chosen.push(candidates[rng.random_range(0..candidates.len())]);

    while chosen.len() < target {
        let mut best_score = i32::MIN;
        let mut best: Vec<TriCoord> = Vec::new();
        for &candidate in candidates {
            if chosen.contains(&candidate) {
                continue;
            }
            let score = chosen.iter().map(|c| c.distance(&candidate)).min().unwrap();
            if score > best_score {
                best_score = score;
                best = vec![candidate];
            } else if score == best_score {
                best.push(candidate);
            }
        }
        let pick = best[rng.random_range(0..best.len())];
        chosen.push(pick);
    }
    chosen
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn returns_min_of_count_and_candidates_len() {
        let candidates: Vec<TriCoord> = (0..5).map(|i| TriCoord::new(i, -i, 1)).collect();
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        assert_eq!(farthest_point_sample(&candidates, 3, &mut rng).len(), 3);
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        assert_eq!(farthest_point_sample(&candidates, 100, &mut rng).len(), 5);
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        assert_eq!(farthest_point_sample(&candidates, 0, &mut rng).len(), 0);
    }

    #[test]
    fn returns_distinct_points_from_candidates() {
        let candidates: Vec<TriCoord> = (0..10).map(|i| TriCoord::new(i, -i, 1)).collect();
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let chosen = farthest_point_sample(&candidates, 6, &mut rng);
        let unique: std::collections::HashSet<_> = chosen.iter().collect();
        assert_eq!(unique.len(), chosen.len());
        for c in &chosen {
            assert!(candidates.contains(c));
        }
    }

    #[test]
    fn prefers_a_spread_out_group_over_a_cluster() {
        // 5 candidates tightly clustered near the origin, 5 far apart around a ring.
        let mut candidates: Vec<TriCoord> = (0..5).map(|i| TriCoord::new(i, -i, 1)).collect();
        candidates.extend([
            TriCoord::new(40, -39, 0),
            TriCoord::new(-40, 41, 0),
            TriCoord::new(40, 0, -39),
            TriCoord::new(-40, 0, 41),
            TriCoord::new(0, 40, -39),
        ]);
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let chosen = farthest_point_sample(&candidates, 5, &mut rng);
        let spread_group = &candidates[5..10];
        let from_spread_group = chosen.iter().filter(|t| spread_group.contains(t)).count();
        assert!(
            from_spread_group >= 4,
            "expected selection to favor the spread-out group, got {chosen:?}"
        );
    }

    #[test]
    fn is_deterministic_for_same_seed() {
        let candidates: Vec<TriCoord> = (0..10).map(|i| TriCoord::new(i, -i, 1)).collect();
        let mut a = ChaCha8Rng::seed_from_u64(9);
        let mut b = ChaCha8Rng::seed_from_u64(9);
        assert_eq!(
            farthest_point_sample(&candidates, 4, &mut a),
            farthest_point_sample(&candidates, 4, &mut b)
        );
    }

    #[test]
    fn differs_across_seeds() {
        let candidates: Vec<TriCoord> = (0..30).map(|i| TriCoord::new(i, -i, 1)).collect();
        let mut a = ChaCha8Rng::seed_from_u64(1);
        let mut b = ChaCha8Rng::seed_from_u64(2);
        assert_ne!(
            farthest_point_sample(&candidates, 4, &mut a),
            farthest_point_sample(&candidates, 4, &mut b)
        );
    }
}
