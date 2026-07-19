//! # world-kernel — the covenant-bound determinism kernel (POC 0)
//!
//! Pure functions from `(seed, params)` to canonical transcript bytes. **No**
//! I/O, time, entropy, threads, or `f32`. Transcendentals go through `libm`
//! only. Every rule here is spelled out in `docs/DETERMINISM.md` and made
//! executable by lints (`clippy.toml`), red-path fixtures (`fixtures/`), and the
//! hashed workload probes — a prose-only rule is a rule we don't actually have
//! (OR5).
//!
//! ## Style forced by the covenant
//!
//! `clippy::arithmetic_side_effects` (denied below) fires on **integer**
//! `+ - * /` — but not on float arithmetic, `% <constant>`, or shifts. So:
//! float math is written naturally (R3), the PRNG uses `wrapping_*` (intentional
//! wrapping, R5), and coordinate/index math uses `checked_*().expect(..)` so an
//! *unintended* overflow panics → nonzero exit → red cell (R5 + release
//! `overflow-checks`).

// R5 made mechanical (see module docs).
#![deny(clippy::arithmetic_side_effects)]
#![forbid(unsafe_code)]

/// Serialized into every transcript header (§5.6). POC 0 == 0. Bumping this is
/// the *only* legitimate way goldens change; a golden change without a bump is
/// definitionally an incident, not a fix (see the mint ceremony).
pub const GENERATOR_VERSION: u32 = 0;

pub mod prng;
pub mod transcript;
pub mod workload;

/// Run the full workload for one seed and return its canonical transcript bytes
/// (§5.6). `gate-runner` SHA-256s exactly these bytes.
pub use workload::run;
