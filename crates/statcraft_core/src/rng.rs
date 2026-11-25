pub use mt19937::Mt19937;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::SystemTime;

mod mt19937 {
    const N: usize = 624;
    const M: usize = 397;
    const MATRIX_A: u32 = 0x9908_b0df;
    const UPPER_MASK: u32 = 0x8000_0000;
    const LOWER_MASK: u32 = 0x7fff_ffff;

    /// Deterministic 32-bit Mersenne Twister (MT19937) PRNG in 100% safe Rust.
    ///
    /// Implements the standard Matsumoto & Nishimura (1998) MT19937 algorithm
    /// with period $2^{19937} - 1$.
    #[derive(Debug, Clone)]
    pub struct Mt19937 {
        mt: [u32; N],
        index: usize,
    }

    impl Mt19937 {
        /// Initializes a new MT19937 generator from a 32-bit seed.
        pub fn new(seed: u32) -> Self {
            let mut mt = [0u32; N];
            mt[0] = seed;
            for i in 1..N {
                let prev = mt[i - 1];
                mt[i] = 1_812_433_253u32
                    .wrapping_mul(prev ^ (prev >> 30))
                    .wrapping_add(i as u32);
            }
            Self { mt, index: N }
        }

        /// Initializes MT19937 from an optional `u64` seed, falling back to
        /// `get_effective_random_seed()` when `None` is provided.
        pub fn from_optional_seed(seed: Option<u64>) -> Self {
            let effective = seed.unwrap_or_else(super::get_effective_random_seed);
            Self::new((effective & 0xffff_ffff) as u32)
        }

        fn twist(&mut self) {
            let mag01 = [0u32, MATRIX_A];
            for kk in 0..(N - M) {
                let y = (self.mt[kk] & UPPER_MASK) | (self.mt[kk + 1] & LOWER_MASK);
                self.mt[kk] = self.mt[kk + M] ^ (y >> 1) ^ mag01[(y & 1) as usize];
            }
            for kk in (N - M)..(N - 1) {
                let y = (self.mt[kk] & UPPER_MASK) | (self.mt[kk + 1] & LOWER_MASK);
                self.mt[kk] = self.mt[kk.wrapping_add(M).wrapping_sub(N)]
                    ^ (y >> 1)
                    ^ mag01[(y & 1) as usize];
            }
            let y = (self.mt[N - 1] & UPPER_MASK) | (self.mt[0] & LOWER_MASK);
            self.mt[N - 1] = self.mt[M - 1] ^ (y >> 1) ^ mag01[(y & 1) as usize];
            self.index = 0;
        }

        /// Generates the next uniformly distributed 32-bit unsigned integer.
        pub fn next_u32(&mut self) -> u32 {
            if self.index >= N {
                self.twist();
            }
            let mut y = self.mt[self.index];
            self.index += 1;

            y ^= y >> 11;
            y ^= (y << 7) & 0x9d2c_5680;
            y ^= (y << 15) & 0xefc6_0000;
            y ^= y >> 18;
            y
        }

        /// Generates the next 64-bit unsigned integer from two 32-bit draws.
        pub fn next_u64(&mut self) -> u64 {
            ((self.next_u32() as u64) << 32) | (self.next_u32() as u64)
        }

        /// Samples a uniform index in `0..upper` without modulo bias using
        /// Lemire-free rejection sampling. Returns `0` if `upper <= 1`.
        pub fn gen_range_usize(&mut self, upper: usize) -> usize {
            if upper <= 1 {
                return 0;
            }
            if upper <= u32::MAX as usize {
                let bound = upper as u32;
                let threshold = bound.wrapping_neg() % bound;
                loop {
                    let r = self.next_u32();
                    if r >= threshold {
                        return (r % bound) as usize;
                    }
                }
            } else {
                let bound = upper as u64;
                let threshold = bound.wrapping_neg() % bound;
                loop {
                    let r = self.next_u64();
                    if r >= threshold {
                        return (r % bound) as usize;
                    }
                }
            }
        }
    }
}

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

    #[test]
    fn test_mt19937_standard_vector_and_determinism() {
        let mut rng1 = Mt19937::new(5489);
        let mut rng2 = Mt19937::from_optional_seed(Some(5489));
        // First 10 outputs of standard MT19937 init_genrand(5489)
        let expected: [u32; 10] = [
            3_499_211_612,
            581_869_302,
            3_890_346_734,
            3_586_334_585,
            545_404_204,
            4_161_255_391,
            3_922_919_429,
            949_333_985,
            2_715_962_298,
            1_323_567_403,
        ];
        for &exp in &expected {
            assert_eq!(rng1.next_u32(), exp);
            assert_eq!(rng2.next_u32(), exp);
        }

        // Cross multiple twist boundaries (> 624 * 2)
        let mut rng_a = Mt19937::new(2_000_000);
        let mut rng_b = Mt19937::new(2_000_000);
        for _ in 0..1500 {
            assert_eq!(rng_a.next_u32(), rng_b.next_u32());
        }
    }

    #[test]
    fn test_mt19937_gen_range_usize() {
        let mut rng = Mt19937::new(42);
        assert_eq!(rng.gen_range_usize(0), 0);
        assert_eq!(rng.gen_range_usize(1), 0);

        let mut seen = [false; 10];
        for _ in 0..500 {
            let v = rng.gen_range_usize(10);
            assert!(v < 10);
            seen[v] = true;
        }
        assert!(seen.iter().all(|&s| s));
    }
}
