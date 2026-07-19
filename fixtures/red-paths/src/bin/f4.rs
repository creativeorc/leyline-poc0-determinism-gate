//! F4 — domain escape (§8, AC6). A NaN reaching serialization must trip the R6
//! finiteness assert → panic → nonzero exit → red cell. Running this binary
//! SHOULD panic; `expect-red.sh` asserts it does and that R6 is named.
//!
//! R6 is the only defense against WASM's one genuinely nondeterministic corner
//! (NaN payloads), so this red path is called out separately in the ACs.

use std::hint::black_box;

use world_kernel::transcript::Writer;

fn main() {
    let mut w = Writer::new();
    // Domain escape: sqrt of a negative → NaN (like `libm::pow(-1.0, 0.5)`).
    let bad = black_box(-1.0_f64).sqrt();
    eprintln!("f4: writing non-finite value {bad:?} to the transcript (expect R6 panic)...");
    w.f64_bits(bad); // panics: "R6 violation: non-finite f64 reached serialization"

    // Only reached if the R6 assert did NOT fire — a failure of the defense.
    println!("F4 FINDING: R6 assert did not fire; a non-finite value was serialized.");
    std::process::exit(3);
}
