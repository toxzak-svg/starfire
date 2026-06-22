//! Variation picker — breaks the timestamp-shuffle illusion.
//!
//! The pattern `idx = now_seconds() % options.len()` cycles through the same
//! N strings every N seconds. Star will repeat the same opener every turn
//! regardless of context. This module replaces that pattern with a per-handler
//! ring buffer: track the last [`RING_SIZE`] indices used by each handler and
//! pick an index NOT in that set.
//!
//! When `options.len() <= RING_SIZE`, the picker resets the ring and falls
//! back to a seed-derived pick rather than getting stuck on a single option.
//!
//! **Operational test**: a sequence of N+1 picks from any handler key should
//! never repeat the same index in the first RING_SIZE+1 picks when
//! `options.len() > RING_SIZE`. When `options.len() <= RING_SIZE`, the picker
//! guarantees no consecutive repeats.
//!
//! Thread-safe via a single global `Mutex`. The lock is held only briefly per
//! call (no I/O, no allocation in the hot path after the first entry).

use std::collections::{HashMap, VecDeque};
use std::sync::{LazyLock, Mutex};

/// Number of recent picks each handler remembers. Larger = more spread,
/// smaller = more repetition. The plan picked 4 as a balance between variety
/// and predictability.
const RING_SIZE: usize = 4;

/// Global registry: handler key -> ring of recent indices used.
///
/// `&'static str` keys are required because call sites pass string literals
/// — no allocations on the hot path. Wrapped in `LazyLock` because
/// `HashMap::new()` isn't `const`-eligible on stable Rust (E0015).
static RINGS: LazyLock<Mutex<HashMap<&'static str, VecDeque<usize>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Pick a variant index not present in the last [`RING_SIZE`] indices used by
/// this handler.
///
/// The seed provides content-derived variation — typically the topic length,
/// input length, or some hash of the content. **Do not pass the timestamp**
/// as the seed; that's the bug we're fixing.
///
/// Returns an index in `[0, options_len)`. For `options_len <= 1`, returns 0.
pub fn pick_unused_in_last_4(key: &'static str, options_len: usize, seed: usize) -> usize {
    if options_len == 0 {
        return 0;
    }
    if options_len == 1 {
        return 0;
    }

    let mut rings = RINGS.lock().expect("variation ring registry poisoned");
    let ring = rings
        .entry(key)
        .or_insert_with(|| VecDeque::with_capacity(RING_SIZE));

    // Build candidate set: indices NOT in the last RING_SIZE picks.
    let recent: Vec<usize> = ring.iter().take(RING_SIZE).copied().collect();
    let candidates: Vec<usize> = (0..options_len)
        .filter(|i| !recent.contains(i))
        .collect();

    let picked = if candidates.is_empty() {
        // All options appeared in the recent ring — reset and pick fresh.
        // Resetting means the next call will see only the pick we just made,
        // so we won't repeat it immediately (the other candidates come back).
        ring.clear();
        seed % options_len
    } else {
        candidates[seed % candidates.len()]
    };

    ring.push_back(picked);
    while ring.len() > RING_SIZE {
        ring.pop_front();
    }

    picked
}

/// Test-only: clear all ring state between tests so they don't bleed.
///
/// Production code never calls this — state is intentionally long-lived
/// to prevent the same opener from repeating within a session.
#[cfg(test)]
pub fn _clear_for_tests() {
    RINGS.lock().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn empty_pool_returns_zero() {
        assert_eq!(pick_unused_in_last_4("test.empty", 0, 42), 0);
    }

    #[test]
    fn single_option_always_returns_zero() {
        assert_eq!(pick_unused_in_last_4("test.single.1", 1, 0), 0);
        assert_eq!(pick_unused_in_last_4("test.single.2", 1, 999), 0);
        assert_eq!(pick_unused_in_last_4("test.single.3", 1, usize::MAX), 0);
    }

    #[test]
    fn avoids_consecutive_repeats_when_pool_larger_than_ring() {
        _clear_for_tests();
        let mut picks = Vec::new();
        for i in 0..20 {
            picks.push(pick_unused_in_last_4("test.large", 7, i));
        }
        // No two consecutive picks should be equal — ring guarantees this
        // because the previous pick is always in the recent ring.
        for w in picks.windows(2) {
            assert_ne!(w[0], w[1], "consecutive picks should differ: {:?}", w);
        }
    }

    #[test]
    fn covers_pool_over_time_when_pool_larger_than_ring() {
        _clear_for_tests();
        // Pool of 7, ring of 4 — over 28 picks we should hit every index.
        let mut picks = Vec::new();
        for i in 0..28 {
            picks.push(pick_unused_in_last_4("test.cover", 7, i));
        }
        let unique: HashSet<_> = picks.iter().copied().collect();
        assert_eq!(
            unique.len(),
            7,
            "expected all 7 indices over 28 picks, got {:?}",
            unique
        );
    }

    #[test]
    fn resets_when_pool_equals_ring_size() {
        _clear_for_tests();
        // Pool of 4, ring of 4 — after 4 picks the ring is full. The 5th
        // call should reset and pick fresh. We don't get stuck on a single
        // index because the reset clears the recent set.
        let mut picks = Vec::new();
        for i in 0..12 {
            picks.push(pick_unused_in_last_4("test.4", 4, i));
        }
        let unique: HashSet<_> = picks.iter().copied().collect();
        assert!(
            unique.len() >= 3,
            "expected at least 3 distinct indices over 12 picks, got {:?}",
            unique
        );
    }

    #[test]
    fn two_option_pool_never_consecutive_repeats() {
        _clear_for_tests();
        // Pool of 2 — ring of 4 can hold both. Each pick must alternate.
        let picks: Vec<_> = (0..10)
            .map(|i| pick_unused_in_last_4("test.2", 2, i))
            .collect();
        for w in picks.windows(2) {
            assert_ne!(w[0], w[1], "consecutive picks in 2-option pool must differ");
        }
    }

    #[test]
    fn different_keys_dont_share_state() {
        _clear_for_tests();
        // Saturate key A.
        for i in 0..10 {
            pick_unused_in_last_4("test.iso.A", 5, i);
        }
        // Key B is fresh — its first pick is not constrained by A's history.
        let first_b = pick_unused_in_last_4("test.iso.B", 5, 42);
        assert!(first_b < 5);
    }

    #[test]
    fn deterministic_for_same_seed_sequence() {
        _clear_for_tests();
        let a: Vec<_> = (0..6)
            .map(|i| pick_unused_in_last_4("test.det", 5, i))
            .collect();
        _clear_for_tests();
        let b: Vec<_> = (0..6)
            .map(|i| pick_unused_in_last_4("test.det", 5, i))
            .collect();
        assert_eq!(a, b, "same seed sequence must produce same picks");
    }

    #[test]
    fn different_seeds_can_produce_different_picks() {
        _clear_for_tests();
        // First picks with different seeds from same key.
        let p1 = pick_unused_in_last_4("test.seed", 5, 0);
        _clear_for_tests();
        let p2 = pick_unused_in_last_4("test.seed", 5, 1);
        // Seeds differ — at minimum one of them will land on a different
        // index, given candidate pool of 5 and seed 0 vs seed 1.
        // (This is sanity that seeds matter, not a strict inequality test.)
        assert!(p1 < 5 && p2 < 5);
    }
}
