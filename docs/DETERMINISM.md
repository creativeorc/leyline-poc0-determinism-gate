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

## Questions the build must answer (Q1–Q8)

Filled in as the corresponding task lands; until then, status is *open*.

| Q | Question | Status / Answer |
|---|----------|-----------------|
| Q1 | Does clippy resolve bans on primitive inherent methods (`f64::sin`) and bare `f32`? | **open** — answered empirically by fixture F5 (T4). Whatever clippy can't express falls back to a CI textual scan, documented here. |
| Q2 | `wasm32-wasip1` vs `wasip2`? | **provisional: wasip1** (simplest module story under wasmtime). Re-verify against current rustc/wasmtime at T5; must not affect numerics. |
| Q3 | ARM runner availability? | **answered:** standalone public repo → free ARM Linux runners; cells B/E/G run as specified (`ubuntu-24.04-arm`), no fallback. |
| Q4 | Toolchain pin. | **rustc 1.93.1** pinned in `rust-toolchain.toml`. wasmtime + Node pins recorded in CI at T6. |
| Q5 | Debug/release parity. | **answered (T3):** `gate-cli --json` produces byte-identical seed digests in debug and release on cell A (x86_64-linux, rustc 1.93.1). R5 holds by construction; release-only is normative henceforth. |
| Q6 | Zero-import instantiation of the std cdylib? | **open** — verified at T5; any panic/abort shims stubbed in `run.mjs` and proven unreachable on green runs. |
| Q7 | `sha2` on wasm? | **open** — confirmed warning-free for both wasm targets at T5; trim features if needed. |
| Q8 | `mul_add` lowering (fused ≠ unfused, both globally consistent)? | **open** — W6 probe 3 records observed bit patterns across all cells (T2/T6). |

---

## What this gate has been shown to catch

Populated in T7 with links to the inverted CI runs demonstrating each red path
(F1 platform-libm leak, F2 iteration-order entropy, F3 silent drift, F4 domain
escape, F5 lint enforcement), including F1's observed cross-target divergence
pattern. *(Empty until the red-path fixtures land.)*
