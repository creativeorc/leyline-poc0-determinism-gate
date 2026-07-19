//! gate-wasm — `wasm32-unknown-unknown` cdylib for the Node/V8 cell (§6.3).
//!
//! No wasm-bindgen (§4.5): the Node harness instantiates this with an empty
//! import object and calls plain `extern "C"` exports, reading the 32-byte
//! digest straight out of exported memory.
//!
//! T1 skeleton. The real exports (T5):
//! ```ignore
//! alloc(len: u32) -> u32
//! run_gate(seed: u64, out_ptr: u32) -> u32  // 0 ok / nonzero error
//! ```
//! which internally run the same 3-repeat self-check as gate-cli.

// Q6 (verify in T5): does the std-built cdylib instantiate with zero imports?
// If panic/abort shims surface, stub them minimally in run.mjs and prove they
// are unreachable on green runs.

/// Placeholder export so the cdylib links and the T1 build is real. Returns the
/// kernel's generator version; replaced by `alloc` / `run_gate` in T5.
#[unsafe(no_mangle)]
pub extern "C" fn generator_version() -> u32 {
    // Exercise both harness dependency edges at T1: the covenant kernel and the
    // shared seed set.
    let _ = gate_runner::SEEDS.len();
    world_kernel::GENERATOR_VERSION
}
