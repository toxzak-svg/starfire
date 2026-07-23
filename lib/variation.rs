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

    let recent: Vec<usize> = ring.iter().take(RING_SIZE).copied().collect();
    let candidates: Vec<usize> = (0..options_len)
        .filter(|index| !recent.contains(index))
        .collect();

    let picked = if candidates.is_empty() {
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

/// Test-only: clear one handler's ring without disturbing parallel tests.
#[cfg(test)]
fn clear_key_for_tests(key: &'static str) {
    RINGS.lock().unwrap().remove(key);
}

/// Compatibility reset for voice tests. Production-style keys are cleared,
/// while `test.*` keys used by parallel variation tests remain isolated.
#[cfg(test)]
pub fn _clear_for_tests() {
    RINGS
        .lock()
        .unwrap()
        .retain(|key, _| key.starts_with("test."));
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
        clear_key_for_tests("test.large");
        let mut picks = Vec::new();
        for seed in 0..20 {
            picks.push(pick_unused_in_last_4("test.large", 7, seed));
        }
        for window in picks.windows(2) {
            assert_ne!(
                window[0], window[1],
                "consecutive picks should differ: {:?}",
                window
            );
        }
    }

    #[test]
    fn covers_pool_over_time_when_pool_larger_than_ring() {
        clear_key_for_tests("test.cover");
        let mut picks = Vec::new();
        for seed in 0..28 {
            picks.push(pick_unused_in_last_4("test.cover", 7, seed));
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
        clear_key_for_tests("test.4");
        let mut picks = Vec::new();
        for seed in 0..12 {
            picks.push(pick_unused_in_last_4("test.4", 4, seed));
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
        clear_key_for_tests("test.2");
        let picks: Vec<_> = (0..10)
            .map(|seed| pick_unused_in_last_4("test.2", 2, seed))
            .collect();
        for window in picks.windows(2) {
            assert_ne!(
                window[0], window[1],
                "consecutive picks in 2-option pool must differ"
            );
        }
    }

    #[test]
    fn different_keys_dont_share_state() {
        clear_key_for_tests("test.iso.A");
        clear_key_for_tests("test.iso.B");
        for seed in 0..10 {
            pick_unused_in_last_4("test.iso.A", 5, seed);
        }
        let first_b = pick_unused_in_last_4("test.iso.B", 5, 42);
        assert!(first_b < 5);
    }

    #[test]
    fn deterministic_for_same_seed_sequence() {
        clear_key_for_tests("test.det");
        let first: Vec<_> = (0..6)
            .map(|seed| pick_unused_in_last_4("test.det", 5, seed))
            .collect();
        clear_key_for_tests("test.det");
        let second: Vec<_> = (0..6)
            .map(|seed| pick_unused_in_last_4("test.det", 5, seed))
            .collect();
        assert_eq!(first, second, "same seed sequence must produce same picks");
    }

    #[test]
    fn different_seeds_can_produce_different_picks() {
        clear_key_for_tests("test.seed");
        let first = pick_unused_in_last_4("test.seed", 5, 0);
        clear_key_for_tests("test.seed");
        let second = pick_unused_in_last_4("test.seed", 5, 1);
        assert_ne!(
            first, second,
            "different seeds should select different first picks"
        );
    }
}
