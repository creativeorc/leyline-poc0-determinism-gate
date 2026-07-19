# POC 0 — The Determinism Gate

Proves the world-generation kernel produces **bit-identical output** across every
build target and host we ship on (native x86-64, native ARM64, WASM under
wasmtime, WASM under V8/Node), and is **demonstrably capable of failing**. This
is both a technical proof and the **governance boundary** for all future
agent-written kernel code: green hash-diff = acceptable, red = rejected.

- **Full build spec:** [`docs/POC0-SPEC.md`](docs/POC0-SPEC.md) (checked in verbatim).
- **The covenant** (rules all identity-bearing code obeys): [`docs/DETERMINISM.md`](docs/DETERMINISM.md).

This is a **standalone public repo**, extracted from the private `leyline`
monorepo so the full CI matrix can run: public repos get free ARM Linux runners
(cells B/E/G) and branch protection (so the gate is *enforced*, not advisory).
It has its own `Cargo.lock`, pinned toolchain (`rust-toolchain.toml`), and
covenant lints (`clippy.toml`). The workload is deliberately representative, not
useful — it exposes no real generation IP.

## Operating rules (non-negotiable — see spec §0)

- **OR1** Goldens are read-only; they change only via the mint ceremony with human approval. A hash mismatch is a *finding to report*, not a bug to re-mint away.
- **OR2** Never weaken enforcement (no `#[allow]` on covenant lints, no `continue-on-error` on gate jobs).
- **OR3** On red: stop, run `compare.py`, report the first divergence. There are no tolerances — equality is bitwise.
- **OR4** The covenant crates' dependency set is fixed (§4.4); additions need sign-off.
- **OR5** Every covenant rule maps to a lint, a red-path fixture, or a hashed probe.
- **OR6** Report spec errors; don't silently substitute semantics that change transcript bytes.

## Layout

```
crates/
  world-kernel/   covenant-bound: SplitMix64 + workloads W1–W6 + transcript writer
  gate-runner/    covenant-clean: seeds -> transcripts -> SHA-256 (+ 3-repeat self-check)
  gate-cli/       harness bin: native + wasm32-wasip1; --json, --transcript-out
  gate-wasm/      cdylib for wasm32-unknown-unknown (Node/V8), no bindgen
fixtures/         red-path F1–F5 (inverted CI jobs)
harness/node/     run.mjs (V8 harness)
harness/compare/  compare.py (fan-in + first-divergence reporting)
goldens/          v0.json — minted only via the ceremony
docs/             DETERMINISM.md (covenant) + POC0-SPEC.md (this spec)
Makefile          `make gate` local loop
```

**CI:** the gate workflows live in `.github/workflows/` (`gate.yml`,
`mint-goldens.yml`) and land in T6/T8.

## Status

**T1 (skeleton) complete:** workspace, toolchain pin, covenant lints, crate homes,
covenant docs, spec checked in. Native build + clippy green.
Next: **T2** — SplitMix64 (with cross-checked KATs), workloads W1–W6, and the
canonical transcript writer. See spec §13 for the T1→T9 sequence.

## Quick start

```bash
cargo build --release        # or: make build
make lint                    # clippy under the covenant config + fmt check
```
