//! F3 — silent drift (§8). A one-constant change to the algorithm keeps every
//! cell agreeing with every other cell, yet changes the digest — so the golden
//! comparison catches it even when cross-cell comparison cannot. This binary
//! computes W1's arithmetic chain (mirroring `world-kernel`) with the real
//! constant and with one constant drifted, and shows the digests differ.
//!
//! The authoritative anchor is `goldens/v0.json` (minted in T8); this fixture
//! proves the digest is *sensitive* to a single-constant edit, which is what
//! makes the golden comparison bite. Exit 0 = demonstrated.

use sha2::{Digest, Sha256};

use world_kernel::prng::{Stream, key};

/// W1's arithmetic chain (§5.3), parameterized on the denominator constant that
/// §5.3 fixes at `2.0`.
fn w1_digest(seed: u64, denom_const: f64) -> [u8; 32] {
    let mut s = Stream::new(key(seed, 1, 0, 0, 0));
    let mut x = 1.0_f64;
    let mut h = Sha256::new();
    for i in 0..256_u32 {
        let a = s.next_f64_12();
        let b = s.next_f64_12();
        x = (a * x + b) / (denom_const + x * x);
        if i % 32 == 31 {
            h.update(x.to_le_bytes());
        }
    }
    h.update(x.to_le_bytes());
    let out = h.finalize();
    let mut d = [0u8; 32];
    d.copy_from_slice(&out);
    d
}

fn main() {
    let seed = 0x0123_4567_89AB_CDEF;
    let real = w1_digest(seed, 2.0);
    let drifted = w1_digest(seed, 2.0 + 1e-7); // an "unreviewed algorithm edit"
    if real != drifted {
        println!(
            "F3 DEMONSTRATED: a one-constant drift changes the digest; the golden \
             comparison catches it even when every cell agrees."
        );
    } else {
        eprintln!("F3 FINDING: the drift did not change the digest.");
        std::process::exit(1);
    }
}
