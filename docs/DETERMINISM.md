# The Determinism Covenant

**Status:** T1 draft (rules complete; empirical answers Q1–Q8 filled in as the
build proceeds). Binds all identity-bearing code — today `world-kernel` and
`gate-runner`; later, the real generation kernel.

The covenant is **executable** (OR5): every rule below maps to at least one of a
lint that fires, a red-path fixture that fails, or a probe whose bytes are
hashed. A rule that is only prose is a rule we don't actually have. Where a rule
is enforced by a mechanism not yet built, its enforcement column names the task
(T-number) that lands it.

---

## Why this exists

Every seed minted for a user's world depends on generation being **bit-identical
across targets for years**. The gate is also the acceptance mechanism for all
future agent-written kernel code: **green hash-diff = acceptable, red =
rejected.** So the gate must be demonstrably capable of failing and impossible
to quietly weaken. Equality is **bitwise** — there are no tolerances.

---

## The rules

### Numeric (R1–R7)

| # | Rule | Enforcement |
|---|------|-------------|
| R1 | **f64 only** for identity-bearing math. No `f32` in covenant crates. | clippy `disallowed-types` (`f32`, pending Q1) + textual-scan fallback; review |
| R2 | **Transcendentals via `libm` crate only** — never the std/core inherent float methods (they route to the *platform* libm; glibc ≠ Apple ≠ musl in final ULPs). | clippy `disallowed-methods` + fixture F1 + W2 hashes |
| R3 | **Allowed core float ops:** `+ - * /`, unary `-`, comparisons, `abs`, `copysign`, `sqrt`, `floor`, `ceil`, `trunc`, `round`, `mul_add`, `to_bits`/`from_bits`, `is_finite`/`is_nan` (asserts only), `total_cmp`, `clamp` (only under the no-NaN invariant R6). Exact or IEEE-correctly-rounded on every target incl. WASM lowerings. | W6 probes (esp. `mul_add`, `round`, `sqrt`) |
| R4 | **Banned float ops:** `f64::min`/`f64::max` (signed-zero order unspecified — use `total_cmp`), `powi` (unspecified expansion), `%` on floats (use `libm::fmod`), any float↔string in kernel code. | clippy `disallowed-methods` + review; `%`-on-float via review/fixture |
| R5 | **Integer overflow is always explicit** — intentional wraps use `wrapping_*`/`Wrapping<T>`; anything else that overflows is a bug. Gives debug/release parity. | crate `#![deny(clippy::arithmetic_side_effects)]` + release `overflow-checks = true` |
| R6 | **No NaN, no infinity ever crosses serialization.** The writer asserts `is_finite()` before every f64 write; a violation panics → nonzero exit → red. (NaN *payloads* are the one genuinely nondeterministic corner of the WASM spec; we don't canonicalize — we outlaw.) | the assert + fixture F4 |
| R7 | **Casts:** f64↔int `as` casts allowed (float→int saturates, int→float rounds-to-nearest — defined & identical everywhere). Pointer/address→int, or hashing addresses, banned. | W6 probes 5–6 + review |

### Structural (R8–R11)

| # | Rule | Enforcement |
|---|------|-------------|
| R8 | **No platform entropy.** Banned in covenant crates: `HashMap`/`HashSet` (randomized `RandomState` iteration), `std::time::*`, `std::thread`, `std::env`, `std::fs`, `std::net`, any env-seeded RNG. Use `Vec`+sort, `BTreeMap`/`BTreeSet`, in-repo SplitMix64. | clippy `disallowed-types`/`disallowed-methods` + fixture F2 |
| R9 | **Total, explicit ordering for every sort.** Comparator is a total order over actual inputs: float keys via `total_cmp`, integer tie-break chain ending in a unique id. `sort_unstable` only when provably total+injective; default stable `sort_by`. | review + W4 hashes |
| R10 | **Fixed iteration & reduction order.** All folds run in index order. No `rayon`, no order-dependent parallel merges. | dependency discipline (OR4) + review |
| R11 | **Serialization is the only output.** Results leave only as the canonical transcript (§5.6 of the spec): LE fixed-width ints, `to_bits()` u64 LE, u64 LE length prefixes, no padding, no text, no `Debug`/`Display` of floats. | transcript writer + review |

### Toolchain (R12–R13)

| # | Rule | Enforcement |
|---|------|-------------|
| R12 | **Pin everything.** Exact stable in `rust-toolchain.toml`; `Cargo.lock` committed; wasmtime + Node versions pinned in CI and recorded in every `hashes.json`. Any pin bump is its own PR whose purpose is re-running the gate. | `rust-toolchain.toml`, committed lock, CI pins, metadata |
| R13 | **No fast-math, no relaxed SIMD, no target-feature games.** No `RUSTFLAGS` enabling `relaxed-simd`; `simd128` OFF for POC 0; no nightly features in covenant crates. | CI env review + no such flags in-repo |

---

## Kernel dependency allowlist (§4.4)

- `world-kernel`: **`libm` only.**
- `gate-runner`: `libm` (transitive) + `sha2`.

That is the entire covenant-bound dependency universe for POC 0. Everything else
(clap, serde_json, …) lives in `gate-cli`/harness only. Adding a dependency to a
covenant crate requires sign-off (OR4).

---

## Recorded decisions & non-promises (§4.5)

- **`std` + lints, not `no_std` (decided for POC 0).** `no_std + alloc` is the
  stronger structural guarantee but adds friction now; the lint set + hash gate +
  fixtures cover the same ground observably. Tracked follow-up: *"Harden kernel to
  `no_std + alloc`"* before POC 2.
- **V8 (Node) is normative, not advisory.** wasmtime alone tests the spec, not the
  ship target. Both runtimes are gate-blocking cells.
- **No wasm-bindgen in POC 0.** The unknown-unknown harness uses plain
  `extern "C"` exports + direct memory reads. Production adopts bindgen later
  without touching kernel numerics.
- **Non-promise — cross-version stability.** The gate proves *cross-target*
  determinism at a *fixed* `GENERATOR_VERSION`. It does **not** promise stability
  across version bumps — that is the pin-per-world mechanism's job. **Nobody,
  human or agent, ever "repairs" an intentional-change mismatch by editing
  goldens outside the ceremony.**
- **Non-promise (temporary) — JavaScriptCore.** Safari/iPad's JSC is a known,
  accepted coverage gap until the physical-iPad work in POC 1. Node's V8 is the
  engine proxy for Chrome/Edge here.

---

## Governance & runner posture (standalone public repo)

This POC was extracted from the private `leyline` monorepo into a standalone
**public** repo (`leyline-poc0-determinism-gate`) precisely so the spec's
governance model works as designed. The workload is representative, not useful,
so nothing real is exposed.

- **Full CI matrix.** Public repos get free ARM Linux runners, so cells B/E/G
  (`aarch64-linux`, native + wasm) run as specified — no macOS fallback, no
  accepted coverage gap.
- **Enforced goldens.** Branch protection + required checks are available. Once
  configured (SETUP.md, T9), CODEOWNERS on `goldens/` is enforced, and a red
  gate *blocks* merge — the standing rule ("no kernel change merges except
  through a green gate") becomes mechanical, not advisory. The **golden-guard CI
  step** (spec §7.4) is the second layer: it fails when `goldens/**` changes
  without a `GENERATOR_VERSION` bump or outside a mint-ceremony PR.
- **Until branch protection is flipped** (an admin step, T9), enforcement is
  review + the golden-guard step; the required-checks config is delivered in
  SETUP.md.

---

## Golden mint ceremony (§7.5)

Goldens change **only** through `poc0-mint-goldens.yml` (`workflow_dispatch`
only):

1. Runs **cell A only**, regenerates `goldens/v0.json` with full provenance
   (`minted_from: cell A`, toolchain, date, commit), opens a PR.
2. That PR must (a) **bump `GENERATOR_VERSION`** if any transcript byte changed
   for existing seeds, (b) pass the full gate — all cells agree with the *new*
   goldens, and (c) be approved by Daniel.
3. Normal CI has **no write path** to `goldens/`.

**Standing rule: a golden change without a version bump is definitionally an
incident, not a fix.**

---

## Cross-target results (evidence)

**T6 — all 7 cells bit-identical, in CI.** The `gate.yml` fan-in reports
`OK: 7 sources agree on all 10 seeds` across the full fleet:

| Cell | Target | Runtime | Host arch |
|---|---|---|---|
| A | `x86_64-unknown-linux-gnu` | native | x86_64 |
| B | `aarch64-unknown-linux-gnu` | native | ARM64 |
| C | `aarch64-apple-darwin` | native | ARM64 (macOS) |
| D | `wasm32-wasip1` | wasmtime 46.0.1 | x86_64 |
| E | `wasm32-wasip1` | wasmtime 46.0.1 | ARM64 |
| F | `wasm32-unknown-unknown` | Node 22 / V8 12.4 | x86_64 |
| G | `wasm32-unknown-unknown` | Node 22 / V8 12.4 | ARM64 |

Every libm transcendental, `mul_add`, subnormal, rounding mode, saturating cast,
and the sort reproduce byte-for-byte across x86_64 **and** ARM64, native **and**
both WASM runtimes — the ARM×WASM quadrant included. Seed `0x0…0` →
`5334a21cf46b7da4d20b5655c067301a8c30b016573e9264ddba39dc9428dcb8`.

First green run: <https://github.com/creativeorc/leyline-poc0-determinism-gate/actions/runs/29673296970>.
*(These digests are dev/CI evidence, not the golden — the authoritative
`goldens/v0.json` is minted via the ceremony, T8, which anchors this exact set.)*

## Questions the build must answer (Q1–Q8)

Filled in as the corresponding task lands; until then, status is *open*.

| Q | Question | Status / Answer |
|---|----------|-----------------|
| Q1 | Does clippy resolve bans on primitive inherent methods (`f64::sin`) and bare `f32`? | **answered (T4): YES, all of them.** Fixture F5 shows clippy 1.93 flags `f64::sin` + `std::time::Instant::now` (`disallowed_methods`), bare `f32` + `std::collections::HashMap` (`disallowed_types`), and integer `+` (`arithmetic_side_effects`). **No textual-scan fallback is needed.** |
| Q2 | `wasm32-wasip1` vs `wasip2`? | **answered (T5): wasip1.** `gate-cli` builds to `wasm32-wasip1` and runs under wasmtime 46.0.1 (`wasmtime run <wasm> --json`; no `--` separator — wasmtime forwards guest args directly). Digests match native bit-for-bit, so the choice does not affect numerics. |
| Q3 | ARM runner availability? | **answered:** standalone public repo → free ARM Linux runners; cells B/E/G run as specified (`ubuntu-24.04-arm`), no fallback. |
| Q4 | Toolchain pin. | **rustc 1.93.1** pinned in `rust-toolchain.toml`. wasmtime + Node pins recorded in CI at T6. |
| Q5 | Debug/release parity. | **answered (T3):** `gate-cli --json` produces byte-identical seed digests in debug and release on cell A (x86_64-linux, rustc 1.93.1). R5 holds by construction; release-only is normative henceforth. |
| Q6 | Zero-import instantiation of the std cdylib? | **answered (T5): YES.** `WebAssembly.Module.imports(gate_wasm.wasm)` is `[]`; `run.mjs` instantiates with `{}`. Exports: `memory, alloc, run_gate, generator_version, seed_count`. No panic/abort shim surfaced — no stub needed. |
| Q7 | `sha2` on wasm? | **answered (T5): YES.** `sha2` builds warning-free for both `wasm32-wasip1` and `wasm32-unknown-unknown` with default features; digests match native. No feature trimming needed. |
| Q8 | `mul_add` lowering (fused ≠ unfused, both globally consistent)? | **open** — W6 probe 3 records observed bit patterns across all cells (T2/T6). |

---

## What this gate has been shown to catch

Populated as each inverted path lands; CI run links added in T7.

- **F5 — lint enforcement (T4; inverted CI job green since T6).** `fixtures/f5-lint/`
  uses each banned construct; `fixtures/f5-lint/expect-red.sh` asserts `cargo
  clippy -- -D warnings` fails *and* reports every one. All five are caught by
  clippy (see Q1). The fixture is excluded from the workspace so it never touches
  the real lint job (OR2). Runs as the `red-path-f5` job in `gate.yml`.
- **F2 — iteration-order entropy (T7).** `fixtures/red-paths/` bin `f2` builds a
  `HashMap` 3× in one process and shows the iteration-order digests diverge —
  exactly what the 3-repeat in-process self-check catches, turning the first cell
  red before the fan-in. CI job `red-path-f234`.
- **F3 — silent drift (T7).** Bin `f3` runs W1's arithmetic chain with one
  denominator constant drifted (`2.0` → `2.0 + 1e-7`); the digest changes. So the
  golden comparison catches an algorithm edit even when every cell agrees. (The
  golden anchor itself is minted in T8.) CI job `red-path-f234`.
- **F4 — domain escape (T7, AC6).** Bin `f4` feeds a NaN (`sqrt(-1)`) to the
  transcript writer; the R6 finiteness assert panics → nonzero exit → red cell.
  R6 is the only defense against WASM's NaN-payload nondeterminism. CI job
  `red-path-f234`.
- **F1 — platform-libm leak (T7).** Bin `f1` uses the std inherent float methods
  (`f64::sin` …) over a transcendental grid. **Observed divergence:** native
  glibc `d7051b29…` vs wasm libm (wasmtime) `195fd4d7…` differ — the std methods
  route to the *platform* libm, so a leak would go red. The `red-path-f1` CI job
  compares cells A (glibc), C (Apple), D (wasm) and asserts they diverge; if they
  ever all agree it fails as a FINDING, not a pass (§8).
