//! Workloads W1–W6 (§5.3–5.4). **Representative, not useful** — the exact
//! semantics here define the transcript bytes; do not "improve" the math.
//!
//! A few covenant-forced rewrites, each bit-equivalent to the spec on the actual
//! input domain (recorded here per OR6, since they change the *operation* even
//! though they cannot change the *bytes*):
//! - `(t + 1) as f64` is written `(t as f64) + 1.0`. For `t ∈ {0,1,2,3}` both are
//!   the exact same f64; the rewrite avoids an integer `+` (lint) with no byte
//!   change. Float multiply association is preserved exactly as written in §5.3.
//! - `i % 2 == 0` is written `i & 1 == 0` (identical for non-negative `i`).

use crate::prng::{Stream, key, mix};
use crate::transcript::{Transcript, Writer, tag};

/// Fixed tiers (§5.1).
const TIERS: [u32; 4] = [0, 1, 2, 3];

/// Fixed chunk coordinates (§5.1). Negative and far-from-origin values are
/// deliberate (sign handling + large-magnitude arithmetic discipline).
const CHUNKS: [(i64, i64); 6] = [
    (0, 0),
    (1, 0),
    (0, -1),
    (-3, 7),
    (1_i64 << 20, -(1_i64 << 20)),
    ((1_i64 << 40) + 1, -(1_i64 << 40) - 1),
];

/// `cx * 16 + i` as `i64`, panicking on the (here impossible) overflow so an
/// unintended one would turn the gate red rather than wrap silently (R5).
#[inline]
fn cell_coord(chunk: i64, within: i64) -> i64 {
    chunk
        .checked_mul(16)
        .and_then(|v| v.checked_add(within))
        .expect("cell coordinate overflow (unexpected for the fixed chunk set)")
}

/// W1 — arithmetic chain (§5.3). `wid=1`, key at `t=0,cx=0,cy=0`.
fn w1(seed: u64) -> Writer {
    let mut s = Stream::new(key(seed, 1, 0, 0, 0));
    let mut w = Writer::new();
    let mut x = 1.0_f64;
    for i in 0..256_u32 {
        let a = s.next_f64_12();
        let b = s.next_f64_12();
        x = (a * x + b) / (2.0 + x * x);
        if i % 32 == 31 {
            w.f64_bits(x);
        }
    }
    // "…and the final x": one more write of the post-loop value (§5.3).
    w.f64_bits(x);
    w
}

/// W2 — transcendental field (§5.3). `wid=2`, per (tier, chunk). 16×16 grid,
/// libm-only transcendentals, then the index-order sum.
fn w2(t: u32, cx: i64, cy: i64) -> Writer {
    let tfac = (t as f64) + 1.0;
    let mut w = Writer::new();
    let mut sum = 0.0_f64;
    for i in 0..16_i64 {
        for j in 0..16_i64 {
            let u = cell_coord(cx, i);
            let v = cell_coord(cy, j);
            // Association preserved exactly as in §5.3: ((coord * 0.001) * tfac).
            let px = (u as f64) * 0.001 * tfac;
            let py = (v as f64) * 0.001 * tfac;
            let val = libm::sin(px) * libm::cos(py)
                + libm::exp(-(px * px + py * py) * 1e-6)
                + libm::atan2(py, px + 1.5);
            w.f64_bits(val);
            sum += val;
        }
    }
    w.f64_bits(sum);
    w
}

/// W3 — feature placement (§5.3). `wid=3`, per (tier, chunk). Sparse features
/// (p ≈ 0.07) with jitter from a fresh stream keyed by `mix(h)`.
fn w3(seed: u64, t: u32, cx: i64, cy: i64) -> Writer {
    // (i, j, kind, jitter_x, jitter_y) in row-major order.
    let mut feats: Vec<(u32, u32, u64, f64, f64)> = Vec::new();
    for i in 0..16_i64 {
        for j in 0..16_i64 {
            let gx = cell_coord(cx, i);
            let gy = cell_coord(cy, j);
            let h = key(seed, 3, t, gx, gy);
            if h < 0x1200_0000_0000_0000 {
                let mut s = Stream::new(mix(h));
                let jitter_x = s.next_f64_01();
                let jitter_y = s.next_f64_01();
                let kind = h % 5;
                feats.push((i as u32, j as u32, kind, jitter_x, jitter_y));
            }
        }
    }
    let mut w = Writer::new();
    w.u64(feats.len() as u64);
    for &(fi, fj, kind, jx, jy) in &feats {
        w.u32(fi);
        w.u32(fj);
        w.u64(kind);
        w.f64_bits(jx);
        w.f64_bits(jy);
    }
    w
}

/// W4 — sort/selection (§5.3). `wid=4`, once per seed (key at `t=0,cx=0,cy=0`).
fn w4(seed: u64) -> Writer {
    struct Cand {
        score: f64,
        x: u64,
        y: u64,
        id: u64,
    }
    let mut s = Stream::new(key(seed, 4, 0, 0, 0));
    let mut cands: Vec<Cand> = Vec::with_capacity(1000);
    for idx in 0..1000_u64 {
        let score = s.next_f64_01();
        let x = s.next_u64();
        let y = s.next_u64();
        cands.push(Cand {
            score,
            x,
            y,
            id: idx,
        });
    }
    // Descending score via total_cmp; tie-break ascending (x, y, id). `id` is
    // unique so the comparator is a total order (R9).
    cands.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then(a.x.cmp(&b.x))
            .then(a.y.cmp(&b.y))
            .then(a.id.cmp(&b.id))
    });
    let mut w = Writer::new();
    for c in cands.iter().take(32) {
        w.u64(c.id);
        w.f64_bits(c.score);
    }
    w
}

/// W5 — reduction (§5.3). `wid=5`, once per seed. Deliberate cancellation in the
/// float accumulator; wrapping bit-sum in the integer accumulator.
fn w5(seed: u64) -> Writer {
    let mut s = Stream::new(key(seed, 5, 0, 0, 0));
    let mut acc_f = 0.0_f64;
    let mut acc_u = 0_u64;
    for i in 0..4096_u64 {
        let v = s.next_f64_01();
        if i & 1 == 0 {
            acc_f += v;
        } else {
            acc_f -= v;
        }
        acc_u = acc_u.wrapping_add(v.to_bits());
    }
    let mut w = Writer::new();
    w.f64_bits(acc_f);
    w.u64(acc_u);
    w
}

/// W6 — edge probes (§5.4). Seed-independent; identical bytes in every
/// transcript. `black_box` forces *runtime* evaluation so we probe the fleet's
/// FPU/lowering, not rustc's compile-time const folding.
fn w6() -> Writer {
    use std::hint::black_box as bb;
    let mut w = Writer::new();

    // 1. subnormal — proves no FTZ/DAZ anywhere in the fleet.
    w.f64_bits(bb(1e-300_f64) * bb(1e-20_f64));

    // 2. signed-zero behavior of core ops (last is +0.0 per IEEE round-to-nearest).
    w.f64_bits(bb(-0.0_f64));
    w.f64_bits(bb(0.0_f64) * bb(-1.0_f64));
    w.f64_bits(bb(-0.0_f64) + bb(0.0_f64));

    // 3. fused vs unfused — must differ, and each be globally consistent (Q8).
    w.f64_bits(bb(0.1_f64).mul_add(bb(10.0_f64), bb(-1.0_f64)));
    w.f64_bits(bb(0.1_f64) * bb(10.0_f64) - bb(1.0_f64));

    // 4. sqrt + rounding modes (round = half-away-from-zero on every target).
    w.f64_bits(bb(2.0_f64).sqrt());
    w.f64_bits(bb(-2.5_f64).floor());
    w.f64_bits(bb(-2.5_f64).trunc());
    w.f64_bits(bb(2.5_f64).round());
    w.f64_bits(bb(-2.5_f64).round());

    // 5. saturating float→int casts (i64 LE): i64::MAX / i64::MIN.
    w.i64(bb(1e300_f64) as i64);
    w.i64(bb(-1e300_f64) as i64);

    // 6. int→float rounding at the 2^53 boundary (expected 1<<53).
    let n = bb(1_i64 << 53).checked_add(1).expect("no overflow");
    w.i64(n as f64 as i64);

    w
}

/// Run the full workload for `seed` and return its canonical transcript (§5.6).
#[must_use]
pub fn run(seed: u64) -> Vec<u8> {
    let mut ts = Transcript::begin(seed);
    ts.section(tag::W1, w1(seed));
    for &t in &TIERS {
        for &(cx, cy) in &CHUNKS {
            ts.section(tag::W2, w2(t, cx, cy));
        }
    }
    for &t in &TIERS {
        for &(cx, cy) in &CHUNKS {
            ts.section(tag::W3, w3(seed, t, cx, cy));
        }
    }
    ts.section(tag::W4, w4(seed));
    ts.section(tag::W5, w5(seed));
    ts.section(tag::W6, w6());
    ts.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_same_seed() {
        assert_eq!(run(0x9E37_79B9_7F4A_7C15), run(0x9E37_79B9_7F4A_7C15));
    }

    #[test]
    fn seeds_differ() {
        assert_ne!(run(0), run(1));
    }

    #[test]
    fn header_is_canonical() {
        let t = run(0x0123_4567_89AB_CDEF);
        assert_eq!(&t[0..4], b"WGT0");
        assert_eq!(&t[4..8], &0u32.to_le_bytes()); // GENERATOR_VERSION
        assert_eq!(&t[8..16], &0x0123_4567_89AB_CDEFu64.to_le_bytes());
    }

    #[test]
    fn all_canonical_seeds_run_finite() {
        // The 10 fixed seeds (§5.1). A non-finite value would panic in the R6
        // assert; reaching the end means every f64 stayed in-domain.
        const SEEDS: [u64; 10] = [
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
        for seed in SEEDS {
            let t = run(seed);
            assert!(!t.is_empty());
        }
    }

    #[test]
    fn w6_is_seed_independent() {
        // The final W6EP section must be byte-identical across seeds. Recompute
        // it directly and confirm it appears at the tail of every transcript.
        let w6_bytes = w6().into_bytes();
        for seed in [0u64, 1, 0xFFFF_FFFF_FFFF_FFFF, 0xDEAD_BEEF_CAFE_BABE] {
            let t = run(seed);
            assert!(t.ends_with(&w6_bytes), "W6 tail differs for seed {seed:#x}");
        }
    }
}
