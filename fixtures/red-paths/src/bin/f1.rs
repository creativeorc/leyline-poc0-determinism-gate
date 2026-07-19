//! F1 — platform-libm leak (§8). Uses the std inherent float methods (`f64::sin`
//! etc.) instead of the `libm` crate — the R2 violation. Prints a SHA-256 over a
//! transcendental grid.
//!
//! Run on multiple cells and compare: the std methods route to the *platform*
//! libm (glibc on Linux, Apple's on macOS, the compiler's fallback on wasm), so
//! the digests are expected to DIVERGE across cells → the gate would go red on a
//! real libm leak. If instead every cell agrees, that is a FINDING to record
//! (std libm matches across our fleet; the fixture needs a sharper trigger), not
//! a pass. The CI job (`red-path-f1`) compares cells and asserts divergence.

use sha2::{Digest, Sha256};

fn main() {
    let mut h = Sha256::new();
    // Arguments large enough that argument reduction / last-ULP differences
    // between platform libms are most likely to show.
    for i in 0..4000_i64 {
        let x = i as f64 * 0.12345;
        let v = x.sin() * x.cos() + (-(x * x) * 1e-6).exp() + (x + 1.5).atan2(x);
        h.update(v.to_le_bytes());
    }
    let out = h.finalize();
    let hex: String = out.iter().map(|b| format!("{b:02x}")).collect();
    println!("{hex}");
}
