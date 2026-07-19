//! gate-wasm — `wasm32-unknown-unknown` cdylib for the Node/V8 cell (§6.3).
//!
//! No wasm-bindgen (§4.5): `harness/node/run.mjs` instantiates this with an empty
//! import object and calls plain `extern "C"` exports, reading the 32-byte digest
//! straight out of exported linear memory. The numeric work is identical to the
//! native/wasmtime cells — only the shim differs — which is the whole point.
//!
//! Q6: this module must instantiate with zero imports. If a future std change
//! surfaces one (e.g. a panic/abort shim), stub it minimally in `run.mjs` and
//! prove it is unreachable on green runs.

use gate_runner::digest_seed;

/// Kernel generator version (read by the harness for its JSON metadata).
#[unsafe(no_mangle)]
pub extern "C" fn generator_version() -> u32 {
    world_kernel::GENERATOR_VERSION
}

/// Number of canonical seeds — lets the harness cross-check its hardcoded list.
#[unsafe(no_mangle)]
pub extern "C" fn seed_count() -> u32 {
    gate_runner::SEEDS.len() as u32
}

/// Leak a `len`-byte buffer and return its address, so the JS harness has a
/// place to receive the digest. Bump/leak is fine for a harness process (§6.3).
#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: u32) -> u32 {
    let mut buf = Vec::<u8>::with_capacity(len as usize);
    let ptr = buf.as_mut_ptr() as u32;
    std::mem::forget(buf);
    ptr
}

/// Compute the digest for `seed` with the same 3-repeat in-process self-check as
/// `gate-cli`, and write its 32 bytes at `out_ptr`. Returns 0 on success, 1 if
/// the in-process repeats disagree (per-process nondeterminism → red cell).
///
/// # Safety
/// `out_ptr` must point to at least 32 writable bytes in this module's memory
/// (obtain it from `alloc(32)`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn run_gate(seed: u64, out_ptr: u32) -> u32 {
    let first = digest_seed(seed);
    // Repeats 1..3: recompute and compare (matches gate-cli's --repeats 3).
    for _ in 0..2 {
        if digest_seed(seed) != first {
            return 1;
        }
    }
    // SAFETY: caller guarantees `out_ptr` covers 32 writable bytes (from alloc).
    unsafe {
        std::ptr::copy_nonoverlapping(first.as_ptr(), out_ptr as *mut u8, 32);
    }
    0
}
