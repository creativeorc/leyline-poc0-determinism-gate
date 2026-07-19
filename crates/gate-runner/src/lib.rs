//! # gate-runner — seeds → transcripts → SHA-256 digests
//!
//! Covenant-clean orchestration (§3): it moves bytes and hashes them; it never
//! *transforms* kernel output. Owns the canonical seed set (§5.1) and the
//! 3-repeat in-process self-check (§6.1), so `gate-cli` and `gate-wasm` share one
//! implementation.

#![deny(clippy::arithmetic_side_effects)]
#![forbid(unsafe_code)]

use std::fmt::Write as _;

use sha2::{Digest, Sha256};

/// Re-exported so harnesses report the kernel's generator version without a
/// direct world-kernel dependency. Lives *inside* the hashed transcript header.
pub use world_kernel::GENERATOR_VERSION;

/// The 10 fixed seeds from §5.1, in canonical order. Their digests are what the
/// gate compares across cells and against the golden file.
pub const SEEDS: [u64; 10] = [
    0x0000_0000_0000_0000,
    0x0000_0000_0000_0001,
    0x0000_0000_FFFF_FFFF,
    0xFFFF_FFFF_FFFF_FFFF,
    0x8000_0000_0000_0000,
    0x9E37_79B9_7F4A_7C15,
    0x0123_4567_89AB_CDEF,
    0xDEAD_BEEF_CAFE_BABE,
    0x5EED_5EED_5EED_5EED,
    0x0F1E_2D3C_4B5A_6978,
];

/// A SHA-256 digest of a transcript.
pub type Digest32 = [u8; 32];

/// The canonical transcript bytes for `seed` (§5.6). Exposed for
/// `--transcript-out` diagnostics; the digest is taken over exactly these bytes.
#[must_use]
pub fn transcript(seed: u64) -> Vec<u8> {
    world_kernel::run(seed)
}

/// SHA-256 over the canonical transcript for `seed`.
#[must_use]
pub fn digest_seed(seed: u64) -> Digest32 {
    let mut hasher = Sha256::new();
    hasher.update(world_kernel::run(seed));
    let out = hasher.finalize();
    let mut d = [0u8; 32];
    d.copy_from_slice(&out);
    d
}

/// Lowercase hex of a digest (§5.6).
#[must_use]
pub fn to_hex(d: &Digest32) -> String {
    let mut s = String::with_capacity(64);
    for b in d {
        // Formatting only — no arithmetic (covenant-clean).
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Raised when a later in-process repeat disagrees with the first — i.e. the run
/// is nondeterministic *within one process* (the fast-fail for things like
/// `RandomState`, ahead of the cross-cell fan-in).
#[derive(Debug, Clone)]
pub struct SelfCheckError {
    pub seed: u64,
    pub repeat: usize,
    pub first: Digest32,
    pub got: Digest32,
}

impl std::fmt::Display for SelfCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "in-process nondeterminism: seed {:#018x} repeat {} gave {} (first run gave {})",
            self.seed,
            self.repeat,
            to_hex(&self.got),
            to_hex(&self.first),
        )
    }
}

impl std::error::Error for SelfCheckError {}

/// Compute digests for all [`SEEDS`], `repeats` times in-process, and error if
/// any repeat disagrees with the first (§6.1). Returns the (seed, digest) pairs
/// in canonical order on success. `repeats` is clamped to at least 1.
///
/// # Errors
/// Returns [`SelfCheckError`] on the first per-process disagreement.
pub fn run_all(repeats: usize) -> Result<Vec<(u64, Digest32)>, SelfCheckError> {
    let baseline: Vec<(u64, Digest32)> = SEEDS.iter().map(|&s| (s, digest_seed(s))).collect();
    // Repeats 1.. re-run and compare; repeat 0 is the baseline itself.
    for repeat in 1..repeats.max(1) {
        for &(seed, first) in &baseline {
            let got = digest_seed(seed);
            if got != first {
                return Err(SelfCheckError {
                    seed,
                    repeat,
                    first,
                    got,
                });
            }
        }
    }
    Ok(baseline)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_is_stable() {
        assert_eq!(digest_seed(0), digest_seed(0));
    }

    #[test]
    fn hex_is_64_lowercase() {
        let h = to_hex(&digest_seed(0xDEAD_BEEF_CAFE_BABE));
        assert_eq!(h.len(), 64);
        assert!(
            h.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase())
        );
    }

    #[test]
    fn self_check_passes_for_deterministic_kernel() {
        let out = run_all(3).expect("kernel must be deterministic in-process");
        assert_eq!(out.len(), SEEDS.len());
        assert_eq!(out[0].0, SEEDS[0]);
    }

    #[test]
    fn distinct_seeds_distinct_digests() {
        assert_ne!(digest_seed(0), digest_seed(1));
    }
}
