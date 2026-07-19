//! # world-kernel — the covenant-bound determinism kernel (POC 0)
//!
//! Pure functions from `(seed, params)` to canonical transcript bytes. **No**
//! I/O, time, entropy, threads, or `f32`. Transcendentals go through `libm`
//! only. Every rule here is spelled out in `docs/DETERMINISM.md` and made
//! executable by lints (`clippy.toml`), red-path fixtures (`fixtures/`), and the
//! hashed workload probes — a prose-only rule is a rule we don't actually have
//! (OR5).
//!
//! ## Square-0 status (T1)
//!
//! This is the T1 skeleton: the crate, the covenant lints, and the module homes
//! exist and compile clean. The actual SplitMix64 PRNG, workloads W1–W6, and the
//! canonical transcript writer land in **T2** (see `docs/POC0-SPEC.md` §5, §13).

// R5 made mechanical: any arithmetic that could overflow must be explicit
// (`wrapping_*` / `checked_*`). Combined with release `overflow-checks = true`,
// this gives debug/release parity by construction.
#![deny(clippy::arithmetic_side_effects)]
#![forbid(unsafe_code)]

/// Serialized into every transcript header (§5.6). POC 0 == 0. Bumping this is
/// the *only* legitimate way goldens change; a golden change without a bump is
/// definitionally an incident, not a fix (see the mint ceremony).
pub const GENERATOR_VERSION: u32 = 0;

/// SplitMix64 PRNG + substream key derivation (§5.2). Known-answer tests are
/// cross-checked against an independent oracle before they are trusted.
pub mod prng {
    // TODO(T2): SplitMix64 `next`/`mix`, `key(..)`, uniform-f64 constructors,
    // and KATs with provenance.
}

/// Workloads W1–W6 (§5.3–5.4): one representative of every operation class the
/// real generator will use. Representative, not useful — do not "improve" it.
pub mod workload {
    // TODO(T2): W1 arithmetic chain, W2 transcendental field, W3 feature
    // placement, W4 sort/selection, W5 reduction, W6 edge probes.
}

/// Canonical transcript serialization (§5.6). Little-endian fixed-width ints,
/// `f64::to_bits()` as u64 LE, u64 LE length prefixes, no padding, no text.
/// Asserts `is_finite()` before every float write (R6).
pub mod transcript {
    // TODO(T2): section tags, header, and the finiteness-asserting writer.
}
