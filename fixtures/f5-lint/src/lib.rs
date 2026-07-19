//! F5 — lint-enforcement red-path fixture (§8, AC4).
//!
//! Every function below uses a construct that is **banned in covenant scope**.
//! Running `cargo clippy -- -D warnings` here MUST fail, with each violation
//! reported. This is the executable proof for **Q1**: does clippy resolve bans
//! on primitive inherent methods (`f64::sin`) and the bare type `f32`? Whatever
//! clippy does *not* catch is documented as a textual-scan fallback in
//! `docs/DETERMINISM.md`, and `expect-red.sh` covers it there instead.
//!
//! This crate is excluded from the workspace and never linked into the real gate
//! (OR2: we do not weaken enforcement — we prove it bites).

// R5 as a hard error at crate level, mirroring the covenant crates.
#![deny(clippy::arithmetic_side_effects)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::time::Instant;

/// R2 — transcendental via the std inherent method instead of `libm`.
pub fn violate_libm(x: f64) -> f64 {
    x.sin()
}

/// R1 — `f32` in covenant scope (type in signature + literal).
pub fn violate_f32() -> f32 {
    1.0_f32
}

/// R8 — `HashMap` (randomized iteration order).
pub fn violate_hashmap() -> HashMap<u32, u32> {
    HashMap::new()
}

/// R5 — unannotated overflow-capable integer arithmetic.
pub fn violate_arithmetic(a: u64, b: u64) -> u64 {
    a + b
}

/// R8 — wall-clock time.
pub fn violate_time() -> Instant {
    Instant::now()
}
