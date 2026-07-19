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

**T1–T3 complete** (native gate works end to end):
- **T1** skeleton — workspace, toolchain pin, covenant lints, crate homes, docs.
- **T2** kernel — SplitMix64 (KATs vs an independent oracle), workloads W1–W6,
  canonical transcript writer with R6 finiteness asserts.
- **T3** native harness — `gate-runner` SHA-256 digests + 3-repeat self-check;
  `gate-cli --json` / `--transcript-out`. Q5 debug/release byte-parity confirmed;
  gate SHA-256 cross-checks against coreutils `sha256sum`.
- **T4** lint wall proven — fixture `fixtures/f5-lint/` + `expect-red.sh` show
  clippy rejects every banned construct. **Q1 answered: all bans resolve in
  clippy 1.93 (no textual-scan fallback).**
- **T5** WASM cells — `gate-cli`→`wasm32-wasip1` under wasmtime, and
  `gate-wasm`→`wasm32-unknown-unknown` under Node/V8 (`harness/node/run.mjs`,
  zero imports). **Cross-target determinism proven: native + wasmtime + V8 give
  bit-identical digests for all 10 seeds.** Q2/Q6/Q7 answered.

- **T6** CI gate — `.github/workflows/gate.yml`: 7-cell matrix (A–G incl. native
  ARM + WASM-on-ARM via free public runners), `compare.py` fan-in with
  first-divergence reporting, forensic metadata, weekly cron, and the F5
  inverted job. `make gate` runs the local subset.

Try it: `make gate` (local cross-check), `make run`, `make f5`.
Next: **T7** — runtime red-paths F1–F4 (prove the gate goes red); **T8** — golden
mint ceremony + guard + CODEOWNERS. See spec §13 for the T1→T9 sequence.

## Quick start

```bash
cargo build --release        # or: make build
make lint                    # clippy under the covenant config + fmt check
```
