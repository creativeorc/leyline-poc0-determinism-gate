//! F2 — iteration-order entropy (§8). `HashMap` iteration order is randomized per
//! instance (`RandomState`), so serializing it makes a transcript nondeterministic
//! *within one process*. This binary rebuilds the map 3 times, hashes the
//! iteration order each time, and reports whether the digests diverge — which is
//! exactly what the gate's 3-repeat in-process self-check would catch (turning
//! the very first cell red, before the fan-in).
//!
//! Exit 0 = red-path demonstrated (digests diverged). Exit 1 = FINDING (no
//! divergence here — fixture proves nothing and needs a sharper trigger).

use std::collections::HashMap;

use sha2::{Digest, Sha256};

fn build_and_digest() -> [u8; 32] {
    let mut m: HashMap<u64, u64> = HashMap::new();
    for i in 0..256_u64 {
        m.insert(i.wrapping_mul(0x9E37_79B9_7F4A_7C15), i);
    }
    let mut h = Sha256::new();
    for (k, v) in &m {
        // Iteration order is RandomState-dependent — the whole point.
        h.update(k.to_le_bytes());
        h.update(v.to_le_bytes());
    }
    let out = h.finalize();
    let mut d = [0u8; 32];
    d.copy_from_slice(&out);
    d
}

fn main() {
    let first = build_and_digest();
    let mut diverged = false;
    for r in 1..3 {
        if build_and_digest() != first {
            diverged = true;
            eprintln!("f2: repeat {r} digest differs from the first (RandomState entropy)");
        }
    }
    if diverged {
        println!(
            "F2 DEMONSTRATED: HashMap iteration order is nondeterministic across \
             in-process rebuilds; the 3-repeat self-check would go RED."
        );
    } else {
        eprintln!("F2 FINDING: no divergence across rebuilds; fixture proves nothing here.");
        std::process::exit(1);
    }
}
