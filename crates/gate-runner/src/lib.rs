//! # gate-runner — seeds → transcripts → SHA-256 digests
//!
//! Covenant-clean orchestration layer (§3): it moves bytes and hashes them; it
//! must never *transform* kernel output. The canonical seed set (§5.1) and the
//! 3-repeat in-process self-check (§6.1) live here so both `gate-cli` and
//! `gate-wasm` share one implementation.
//!
//! ## Square-0 status (T1)
//!
//! Skeleton only. Digest orchestration and the self-check land in **T3** once
//! the kernel's transcript writer exists (T2).

#![forbid(unsafe_code)]

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

// TODO(T3): `digest_seed(seed) -> [u8; 32]` (SHA-256 over the kernel transcript)
// and `run_all()` with the 3-repeat self-check that fails fast on per-process
// nondeterminism (e.g. RandomState).
