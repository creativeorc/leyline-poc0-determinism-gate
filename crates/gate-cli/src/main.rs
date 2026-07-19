//! gate-cli — native + `wasm32-wasip1` harness (§6.1).
//!
//! T1 skeleton. Full behavior (T3): run all seeds 3x in-process, fail on any
//! per-process disagreement, and print `--json` (generator_version, target,
//! runtime, toolchain, metadata, per-seed SHA-256). `--transcript-out <dir>`
//! dumps raw transcripts for diagnosis.

fn main() {
    // The seed set is already fixed and shared; prove the wiring compiles and
    // runs on native and under wasmtime before any numeric code exists.
    println!(
        "gate-cli skeleton (T1): {} canonical seeds wired.",
        gate_runner::SEEDS.len(),
    );
    println!("note: digest orchestration + --json land in T3 (see docs/POC0-SPEC.md).");
}
