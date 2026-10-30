use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::SystemTime;

static HAS_FIXED_SEED: AtomicBool = AtomicBool::new(false);
static GLOBAL_SEED: AtomicU64 = AtomicU64::new(2_000_000);

/// Sets the global random number generator seed to a fixed value.
pub fn set_global_random_seed(seed: u64) {
    GLOBAL_SEED.store(seed, Ordering::SeqCst);
    HAS_FIXED_SEED.store(true, Ordering::SeqCst);
}

/// Clears the fixed random seed, returning the generator to random initialization mode.
pub fn clear_global_random_seed() {
    HAS_FIXED_SEED.store(false, Ordering::SeqCst);
}

/// Returns the current fixed random seed, or `None` if running in random initialization mode.
pub fn get_global_random_seed() -> Option<u64> {
    if HAS_FIXED_SEED.load(Ordering::SeqCst) {
        Some(GLOBAL_SEED.load(Ordering::SeqCst))
    } else {
        None
    }
}

/// Returns an effective seed for RNG operations:
/// either the fixed seed if configured, or a clock-derived seed between 1 and 2,000,000,000.
pub fn get_effective_random_seed() -> u64 {
    if let Some(seed) = get_global_random_seed() {
        seed
    } else {
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(2_000_000);
        (nanos % 2_000_000_000).max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_random_seed_lifecycle() {
        clear_global_random_seed();
        assert_eq!(get_global_random_seed(), None);

        let effective_random = get_effective_random_seed();
        assert!((1..=2_000_000_000).contains(&effective_random));

        set_global_random_seed(123_456);
        assert_eq!(get_global_random_seed(), Some(123_456));
        assert_eq!(get_effective_random_seed(), 123_456);

        set_global_random_seed(2_000_000);
        assert_eq!(get_global_random_seed(), Some(2_000_000));
        assert_eq!(get_effective_random_seed(), 2_000_000);

        clear_global_random_seed();
        assert_eq!(get_global_random_seed(), None);
    }
}
