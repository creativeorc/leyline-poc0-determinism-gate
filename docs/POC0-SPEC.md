# POC 0 — The Determinism Gate: Build Specification

> **FROZEN — as-issued.** This is the original build spec, kept unchanged as the
> historical record. The **living** covenant is `docs/DETERMINISM.md`; where the
> build diverged from this spec, the deviation notes below and that doc record
> why. Deviations recorded so far:
> - Standalone **public** repo (not a new repo per Daniel's later choice) — §3 note.
> - CI matrix **extended from 7 cells (A–G) to 9 (A–I)**: added Bun cells H/I that
>   run the wasm under JavaScriptCore/WebKit (the iPad's WKWebView engine),
>   converting §4.5's temporary JSC non-promise into CI evidence. Where this spec
>   says "7 cells" / "seven cells" (e.g. §7.1, §7.5, AC1), read "9 (A–I)".

**Project:** Fantasy World GIS · **Doc status:** Ready for implementation · **Audience:** coding agent team
**Owner / approver:** Daniel · **Upstream context:** Product & Technical Plan §3.3, §4 ("Determinism gate — hard prerequisite"), §9 (POC ladder)
**Estimated effort:** 3–5 focused days

---

## 0. Read this first — purpose and operating rules

POC 0 builds the **determinism gate**: a CI system that proves the world-generation kernel produces **bit-identical output** across every build target and host we ship on (native x86-64, native ARM64, WASM under wasmtime, WASM under V8). It has two jobs, and both are in scope:

1. **Technical proof.** The Rust → WASM toolchain, with the covenant rules below, yields byte-for-byte reproducible generation. Every seed minted for a user's world depends on this property holding for years.
2. **Governance boundary.** This gate is the acceptance mechanism for all future agent-written kernel code: green hash-diff = acceptable, red = rejected. That means the gate itself must be *demonstrably capable of failing* and *impossible to quietly weaken*. POC 0 is the fence that makes the rest of the pasture safe.

Nothing in this POC is algorithmically hard. Everything in it is **exacting**. The failure mode we are defending against is not "can't build it" — it is a gate that goes green while being weaker than we believe. Read the acceptance criteria (§12) as the definition of the work, not as a checklist applied afterward.

### Operating rules for the agent team (non-negotiable)

- **OR1 — Goldens are read-only.** Never create, modify, or delete anything under `goldens/` in a normal PR. Goldens change only via the mint ceremony (§7.5) with explicit human approval. A hash mismatch is **a finding to report, not a bug to fix** by re-minting.
- **OR2 — Never weaken enforcement.** No `#[allow(...)]` on covenant lints, no loosening of `clippy.toml`, no skipping or `continue-on-error` on gate jobs (outside the deliberately inverted red-path jobs in §8). If a rule seems wrong or unworkable, stop and raise it as a question with evidence.
- **OR3 — On red, stop and diagnose.** If any cell disagrees with any other cell or with the golden: halt, run the transcript-diff tooling (§6.4), attach artifacts, and report the first point of divergence. Do not "fix" a mismatch by changing serialization, reordering operations, or adjusting tolerances. There are no tolerances. Equality is bitwise.
- **OR4 — Dependency discipline.** The kernel crate's dependency set is enumerated in §4.4. Adding any dependency to a covenant-bound crate requires sign-off. Harness/CI code may use ordinary dependencies.
- **OR5 — The covenant is executable.** Every rule in `DETERMINISM.md` must map to at least one of: a lint that fires, a red-path fixture that fails, or a probe in the workload whose bits are hashed. A rule that is only prose is a rule we don't actually have.
- **OR6 — Report spec errors, don't improvise.** If any instruction in this document turns out to be technically wrong (an API changed, a target renamed, a lint path doesn't resolve), report the discrepancy and the proposed correction. Do not silently substitute semantics that change transcript bytes.

---

## 1. Objective and non-goals

**Objective.** A repository containing (a) a small covenant-bound Rust kernel exercising every operation class the real generator will use, (b) native and WASM harnesses that hash its output canonically, (c) a CI matrix proving bitwise equality across all cells against committed goldens, (d) red-path fixtures proving the gate catches each class of violation, and (e) the covenant document and golden-minting ceremony that govern all future kernel work.

**Non-goals — explicitly out of scope for POC 0:**

- Real terrain/settlement generation, noise quality, or any visual output. The workload is representative, not useful.
- Performance targets of any kind. Record timings as informational metadata only.
- Rendering, WebGPU, SharedArrayBuffer, COOP/COEP headers. (No shared memory is used here; those constraints enter at POC 1.)
- Browser-in-CI. Node's V8 is the engine proxy for Chrome/Edge. JavaScriptCore (Safari/iPad) coverage is a **known, accepted gap** until the physical-iPad work in POC 1 — record this in `DETERMINISM.md` §Non-promises.
- Threading/parallel generation. The kernel is single-threaded pure functions for now; parallelism is a later, gated change.
- `no_std`. We start with `std` + deny-lints (rationale in §4.5). Converting the kernel to `no_std + alloc` is a tracked hardening follow-up, not POC 0 work.

---

## 2. Definitions

| Term | Meaning |
|---|---|
| **Kernel** | The covenant-bound crate `world-kernel`: pure functions from `(seed, params)` to bytes. No I/O, no time, no entropy, no threads. |
| **Transcript** | The canonical byte stream a kernel run emits for one seed (format in §5.6). The thing that gets hashed. |
| **Golden** | The committed SHA-256 hex digest of a transcript for a given `(GENERATOR_VERSION, seed)`, minted via the ceremony in §7.5. |
| **Cell** | One (build target × runtime × host arch) combination in the CI matrix (§7.1). |
| **Covenant** | The ruleset in §4, published as `docs/DETERMINISM.md`, enforced by lints + fixtures + the hash gate. |
| **GENERATOR_VERSION** | A `u32` const in the kernel, serialized into every transcript header. Bumping it is the only legitimate way goldens change. POC 0 uses version `0`. |

---

## 3. Repository layout

```
/  (new repo — name/visibility per Daniel, see §11)
├── Cargo.toml                  # workspace
├── rust-toolchain.toml         # pinned stable (see Q4)
├── Cargo.lock                  # committed
├── clippy.toml                 # covenant deny-lists (§9)
├── CODEOWNERS                  # goldens/** and docs/DETERMINISM.md require Daniel
├── crates/
│   ├── world-kernel/           # covenant-bound. Workloads W1–W6, transcript writer.
│   ├── gate-runner/            # covenant-clean lib: orchestrates seeds → SHA-256 digests (uses sha2)
│   ├── gate-cli/               # bin. Builds native AND wasm32-wasip1. `--json`, `--transcript-out`
│   └── gate-wasm/              # cdylib for wasm32-unknown-unknown (Node/V8 harness). No wasm-bindgen.
├── fixtures/                   # red-path crates F1–F5 (§8) — never linked into the real gate
├── harness/
│   ├── node/run.mjs            # loads gate-wasm, emits hashes.json (same shape as gate-cli --json)
│   └── compare/compare.py      # fan-in: pairwise cell equality + golden equality + first-divergence report
├── goldens/
│   └── v0.json                 # { generator_version, minted_from, toolchain, date, seeds: {hex_seed: hex_sha256} }
├── docs/
│   ├── DETERMINISM.md          # the covenant (§4) + non-promises + mint ceremony
│   └── POC0-SPEC.md            # this document, checked in verbatim
├── .github/workflows/
│   ├── gate.yml                # PR + push + weekly cron: full matrix + fan-in + red-path + lint jobs
│   └── mint-goldens.yml        # workflow_dispatch only; opens a PR; human approval required
└── Makefile                    # `make gate` local subset (§6.5)
```

Notes: `world-kernel` and `gate-runner` are covenant-bound (lints apply). `gate-cli`, `gate-wasm`, `harness/`, and `fixtures/` are not, but must not transform kernel output — they move bytes and hash them.

> **POC 0 note (recorded):** this POC is the standalone **public** repo
> `leyline-poc0-determinism-gate`, extracted from the private `leyline` monorepo
> so the full A–G matrix (free ARM Linux runners) and enforced branch protection
> are available — matching this layout as written. See `DETERMINISM.md`
> "Governance & runner posture".

---

## 4. The Determinism Covenant (contents of `docs/DETERMINISM.md`)

These rules bind all identity-bearing code (today: `world-kernel`, `gate-runner`; later: the real generation kernel). Each rule lists its enforcement. Agents: per OR5, if you write a rule you must also write its probe, lint, or fixture.

### 4.1 Numeric rules

- **R1 — f64 only for identity-bearing math.** No `f32` anywhere in covenant-bound crates. (`f32` exists only downstream of identity, GPU-side, out of scope here.) *Enforced:* clippy `disallowed-types` on `f32` in kernel crates (see Q1 for syntax verification); code review.
- **R2 — Transcendentals via the `libm` crate only.** `libm::sin/cos/tan/asin/acos/atan/atan2/exp/exp2/log/log2/log10/pow/fmod/...` — never the inherent `std`/`core` float methods, which route to the *platform* libm (glibc ≠ Apple ≠ musl in final ULPs). *Enforced:* clippy `disallowed-methods` (§9) + red fixture F1 + workload W2 hashes.
- **R3 — Allowed core float operators/methods:** `+ - * /`, unary `-`, comparisons, `abs`, `copysign`, `sqrt`, `floor`, `ceil`, `trunc`, `round`, `mul_add`, `to_bits`/`from_bits`, `is_finite`/`is_nan` (assertions only), `total_cmp`, `clamp` **only** with the no-NaN invariant (R6). These are exact or IEEE-correctly-rounded on every target, including their WASM lowerings. `mul_add` is a correctly-rounded fused op everywhere (hardware FMA natively, correctly-rounded software lowering on wasm) — allowed, and probed in W6 so the claim is executable, not folklore.
- **R4 — Banned float operations:** `f64::min` / `f64::max` (Rust documents their signed-zero ordering as unspecified — a real cross-target hazard; select via `total_cmp` or explicit comparison instead); `powi` (unspecified expansion — use an explicit loop or `libm::pow`); the `%` operator on floats (use `libm::fmod` if remainder is ever needed); any float→string or string→float conversion in kernel code (bits only, never text). *Enforced:* clippy `disallowed-methods` + review; the `%`-on-float ban via review + fixture if clippy can't express it.
- **R5 — Integer overflow is always explicit.** All intentionally wrapping arithmetic uses `wrapping_*` / `Wrapping<T>`; anything else that could overflow is a bug. This is what makes debug and release builds behave identically. *Enforced:* `clippy::arithmetic_side_effects` denied in kernel crates; `overflow-checks = true` in the **release** profile for the workspace (perf is irrelevant at POC 0), so unintended overflow turns the gate red instead of silently wrapping.
- **R6 — No NaN, no infinity, ever crosses the serialization boundary.** The transcript writer asserts `is_finite()` on every f64 before writing its bits. A violation panics, the process exits nonzero, the cell fails, the gate is red. Kernel code must keep inputs in-domain. (Background: NaN *payloads* are the one place the WASM spec is genuinely nondeterministic; we don't canonicalize NaNs — we outlaw them.) *Enforced:* the assert itself + red fixture F4.
- **R7 — Casts.** `as` casts between f64 and integers are allowed (float→int saturates, int→float rounds-to-nearest — both fully defined and identical on all targets; probed in W6). Casting pointers/addresses to integers, or hashing addresses, is banned.

### 4.2 Structural rules

- **R8 — No platform entropy.** Banned in covenant crates: `std::collections::HashMap` / `HashSet` (per-process `RandomState` randomizes iteration order), `std::time::*` (`Instant`, `SystemTime`), `std::thread`, `std::env`, `std::fs`, `std::net`, any RNG seeded from the environment. Replacements: `Vec` + sort, `BTreeMap`/`BTreeSet`, the in-repo SplitMix64 (§5.2). *Enforced:* clippy `disallowed-types` / `disallowed-methods` + red fixture F2.
- **R9 — Total, explicit ordering for every sort.** Sorts use `sort_by` with a comparator that is a total order over the actual inputs: float keys via `total_cmp`, with an integer tie-break chain ending in a unique id. `sort_unstable` is allowed only when the comparator is provably total and injective on inputs — default to stable `sort_by`. *Enforced:* review + W4 hashes.
- **R10 — Fixed iteration and reduction order.** All folds/reductions run in index order. No `rayon`, no order-dependent parallel merges (parallelism is a future, separately-gated change). *Enforced:* dependency discipline (OR4) + review.
- **R11 — Serialization is the only output.** Kernel results leave only as the canonical transcript (§5.6): little-endian fixed-width integers, `f64::to_bits()` as u64 LE, u64 LE length prefixes, no padding, no text formatting, no `Debug`/`Display` of floats.

### 4.3 Toolchain rules

- **R12 — Pinned everything.** `rust-toolchain.toml` pins an exact stable release; `Cargo.lock` is committed; wasmtime and Node versions are pinned in CI and recorded in every `hashes.json`. Any pin bump is its own PR whose entire purpose is re-running the gate; if goldens still match (expected — the covenant uses only strictly-specified semantics), merge; if not, that is a *finding* (see the ceremony, §7.5).
- **R13 — No fast-math, no relaxed SIMD, no target-feature games.** No `RUSTFLAGS` enabling `relaxed-simd`; do not enable `simd128` for POC 0 (plain simd128 is spec-deterministic and may be enabled later behind the gate, but it is a variable we exclude now). No nightly features in covenant crates.

### 4.4 Kernel dependency allowlist

`world-kernel`: **`libm` only.** `gate-runner`: `libm` (transitively) + `sha2`. That's the whole covenant-bound dependency universe for POC 0. (`sha2`'s optional hardware acceleration doesn't matter — SHA-256 is exact regardless of implementation path.) Everything else (serde_json, clap, etc.) lives in `gate-cli`/harness only.

### 4.5 Recorded decisions and non-promises

- **`std` + lints, not `no_std` (decided for POC 0).** `no_std + alloc` is the stronger structural guarantee (platform entropy unreachable by construction) but adds friction while the team is new to Rust; the lint set + hash gate + fixtures cover the same ground observably. File a tracked issue: "Harden kernel to `no_std + alloc`" for revisit before POC 2.
- **V8 is normative, not advisory.** wasmtime alone would test the spec, not the ship target. Both runtimes are gate-blocking cells.
- **No wasm-bindgen in POC 0.** The unknown-unknown harness uses plain `extern "C"` exports + direct memory reads (§6.3). Binding glue is irrelevant to numeric determinism and only adds moving parts; production will adopt bindgen later without touching kernel numerics.
- **Non-promise: cross-version stability.** The gate proves *cross-target* determinism at a *fixed* generator version. It does **not** promise stability across `GENERATOR_VERSION` bumps — that is the pin-per-world mechanism's job (product plan §3.4). Write this in `DETERMINISM.md` in exactly these terms so nobody, human or agent, ever "repairs" an intentional-change mismatch by editing goldens outside the ceremony.
- **Non-promise (temporary): JavaScriptCore.** See §1 non-goals.

---

## 5. Workload specification

The workload is **representative, not useful**: its only purpose is to exercise every operation class the real generator will rely on, so the gate's green actually covers the code we'll write later. Do not "improve" the math; the exact semantics below define the transcript bytes.

### 5.1 Parameters

- **Seeds (10, fixed):**
  `0x0000000000000000, 0x0000000000000001, 0x00000000FFFFFFFF, 0xFFFFFFFFFFFFFFFF, 0x8000000000000000, 0x9E3779B97F4A7C15, 0x0123456789ABCDEF, 0xDEADBEEFCAFEBABE, 0x5EED5EED5EED5EED, 0x0F1E2D3C4B5A6978`
- **Tiers:** `t ∈ {0, 1, 2, 3}` (u32).
- **Chunk coordinates per tier (i64 pairs, fixed):** `(0,0), (1,0), (0,-1), (-3,7), (2^20, -2^20), (2^40 + 1, -(2^40) - 1)`. Negative and large-magnitude coordinates are deliberate: they smoke-test sign handling and far-from-origin arithmetic discipline (real precision work is POC 1/2's problem, but the gate should already be exercising the territory).

### 5.2 PRNG — SplitMix64 (implemented in-repo, ~15 lines, no dependency)

```
next(state: &mut u64) -> u64:
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15)
    z = *state
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9)
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB)
    return z ^ (z >> 31)
```

Define `mix(x: u64) -> u64` as the finalizer alone (the three lines after the add, applied to `x`). **Substream key derivation** for workload id `wid: u64`, tier `t`, chunk `(cx, cy)`:

```
key(seed, wid, t, cx, cy) = mix( mix( mix( mix( mix(seed) ^ wid ) ^ (t as u64) ) ^ (cx as u64) ) ^ (cy as u64) )
```

(`as u64` on i64 is the two's-complement bit reinterpretation — fine and deterministic.) A stream's state starts at its key. **Known-answer tests:** generate reference vectors for `next` from Vigna's public-domain reference C implementation of SplitMix64, commit them as unit tests with a provenance comment (URL + retrieval date). Do not trust remembered constants — including any you think you know.

**Uniform f64 in [1.0, 2.0):** `f64::from_bits(0x3FF0_0000_0000_0000 | (r >> 12))`. **[0.0, 1.0):** the same minus `1.0`. These constructions are exact bit manipulation — no rounding path.

### 5.3 Workloads W1–W5 (per seed; W1/W4/W5 run once per seed, W2/W3 run per (tier, chunk))

- **W1 — Arithmetic chain** (`wid=1`, key uses `t=0, cx=0, cy=0`). `x = 1.0`; for `i in 0..256`: draw `a_i, b_i ∈ [1,2)`; `x = (a_i * x + b_i) / (2.0 + x * x)`. Serialize `x.to_bits()` at every `i % 32 == 31` and the final `x`. (Bounded: |x| grows at most linearly; always finite.)
- **W2 — Transcendental field** (`wid=2`, per tier × chunk). 16×16 grid; for cell `(i, j)` with `u = cx * 16 + i`, `v = cy * 16 + j` (i64 math, then `as f64`): `px = u as f64 * 0.001 * (t + 1) as f64`, `py = v as f64 * 0.001 * (t + 1) as f64`; `val = libm::sin(px) * libm::cos(py) + libm::exp(-(px*px + py*py) * 1e-6) + libm::atan2(py, px + 1.5)`. Serialize all 256 `val.to_bits()` in row-major order, then a fixed-order (index-order) sum's bits. Large `|px|` (from the 2^40 chunks) is intentional — libm's own argument reduction is source-fixed, so wildly-reduced sin of a huge argument is still bit-identical everywhere, and the gate proves it.
- **W3 — Feature placement** (`wid=3`, per tier × chunk). For each of the 256 cells: `h = key(seed, 3, t, cx*16+i, cy*16+j)`; if `h < 0x1200_0000_0000_0000` (p ≈ 0.07): a feature exists with `jitter_x, jitter_y ∈ [0,1)` drawn from a fresh stream keyed by `mix(h)`, and `kind = h % 5`. Serialize `count: u64`, then per feature in row-major order: `i: u32, j: u32, kind: u64, jitter_x bits, jitter_y bits`.
- **W4 — Sort/selection** (`wid=4`). Draw 1000 candidates: `score ∈ [0,1)`, `x: u64`, `y: u64`, `id = index as u64`. Stable `sort_by`: descending `score` via `total_cmp`, tie-break ascending `(x, y, id)`. Serialize the top 32 as `id, score bits` pairs.
- **W5 — Reduction** (`wid=5`). Draw 4096 values `v ∈ [0,1)`; fixed index-order fold of `acc_f = acc_f + if i % 2 == 0 { v } else { -v }` (deliberate cancellation) and `acc_u = acc_u.wrapping_add(v.to_bits())`. Serialize both.

### 5.4 W6 — Edge probes (seed-independent; identical bytes in every transcript)

These turn every "known non-issue" claim into hashed evidence. Serialize, in this order:

1. `(1e-300_f64 * 1e-20).to_bits()` — a **subnormal**; proves no FTZ/DAZ anywhere in the fleet.
2. `(-0.0_f64).to_bits()`, `(0.0_f64 * -1.0).to_bits()`, `(-0.0_f64 + 0.0).to_bits()` — signed-zero behavior of core ops (the last is +0.0 per IEEE round-to-nearest).
3. `0.1_f64.mul_add(10.0, -1.0).to_bits()` **and** `(0.1_f64 * 10.0 - 1.0).to_bits()` — fused vs. unfused differ; both must be stable per-target-class and identical across all cells.
4. `2.0_f64.sqrt().to_bits()`, `(-2.5_f64).floor().to_bits()`, `(-2.5_f64).trunc().to_bits()`, `(2.5_f64).round().to_bits()`, `(-2.5_f64).round().to_bits()` — locks Rust's round-half-away-from-zero semantics on every target (WASM's native `f64.nearest` is half-to-even; rustc must and does compensate — we verify).
5. `(1e300_f64 as i64)` and `(-1e300_f64 as i64)` as i64 LE — saturating casts (`i64::MAX` / `i64::MIN`).
6. `(((1_i64 << 53) + 1) as f64 as i64)` as i64 LE — int→float rounding at the 2^53 boundary (expected `1 << 53`).

### 5.5 What deliberately is NOT in the workload

No `HashMap` (banned), no threads, no time, no `f32`, no NaN/inf construction, no text formatting of numbers. If you find yourself needing any of these to implement §5.3, you have misread the spec — stop (OR6).

### 5.6 Transcript format (canonical serialization)

```
header:  magic "WGT0" (4 bytes ASCII) · GENERATOR_VERSION u32 LE · seed u64 LE
then, in fixed order:
  W1 section · W2 sections (t asc, then chunk list order §5.1) · W3 sections (same order) · W4 · W5 · W6
section: tag 4 bytes ASCII ("W1AR","W2TR","W3HF","W4SO","W5RD","W6EP") · payload_len u64 LE · payload
```

All integers little-endian fixed-width; every f64 as `to_bits()` u64 LE; collections length-prefixed u64 LE; no alignment padding. The writer asserts `is_finite()` before every float write (R6). **Digest = SHA-256 over the full transcript, lowercase hex.** `GENERATOR_VERSION` lives *inside* the hashed bytes, so goldens mechanically change when it bumps.

---

## 6. Harnesses

### 6.1 `gate-cli` (native + wasm32-wasip1)

One bin crate, two build targets. `gate-cli --json` prints:

```json
{ "generator_version": 0, "target": "...", "runtime": "native|wasmtime",
  "toolchain": "<rustc -Vv summary>", "runtime_version": "...", "host_arch": "...",
  "repeats": 3, "seeds": { "0x0000000000000000": "<sha256 hex>", ... } }
```

It runs the full seed set **3 times in-process** and errors out if any repeat disagrees with the first (this is what catches per-process nondeterminism like `RandomState` fast, without waiting for the fan-in). `--transcript-out <dir>` additionally writes raw transcripts (`seed_<hex>.bin`) for diagnosis. Under wasmtime: `wasmtime run gate-cli.wasm -- --json` (stdout via WASI; no filesystem needed unless `--transcript-out`, then grant a dir).

### 6.2 Build targets

Native: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `aarch64-apple-darwin` (each built on its own runner — no cross-compilation needed). WASM: `wasm32-wasip1` (wasmtime) and `wasm32-unknown-unknown` (Node). Yes, that is two distinct WASM builds — identical numeric code, different shims — and that is a feature: both are matrix cells (see Q2 on wasip1 vs wasip2).

### 6.3 `gate-wasm` + Node harness (wasm32-unknown-unknown, no bindgen)

Exports, plain `extern "C"` / `#[no_mangle]`:

```
alloc(len: u32) -> u32                         // bump/leak allocator is fine for a harness
run_gate(seed: u64, out_ptr: u32) -> u32       // writes 32-byte digest at out_ptr; returns 0 ok / nonzero error
                                               // internally performs the 3-repeat self-check like gate-cli
```

`harness/node/run.mjs`: read the `.wasm`, `WebAssembly.instantiate(bytes, {})` (the module must instantiate with **zero imports** — if the std build demands any, see Q6), loop the seed list passing `seed` as a `BigInt`, read 32 bytes from exported memory, hex-encode, emit the same JSON shape as `gate-cli --json` with `"runtime": "node"` and `runtime_version` = `process.version` + `process.versions.v8`. Plain Node, no bundler, no npm install.

### 6.4 Diagnosability — required, not optional

`harness/compare/compare.py`: given N `hashes.json` files + the golden file → report full pairwise agreement or, on mismatch, exactly which (cell, seed) pairs disagree. Given two transcript files → report the **first divergent section tag and byte offset**. On any red fan-in, CI uploads all transcripts as artifacts and prints the compare output. A gate that says only "red" generates the 2 a.m. phone call this document exists to prevent; a gate that says "cell F, seed 0xDEAD…, section W2TR, byte 1832" does not.

### 6.5 Local loop

`make gate`: build native + both wasm targets, run native + Node (+ wasmtime if installed), run `compare.py` against goldens. Agents iterate locally; CI is confirmation, not the debugger.

---

## 7. CI design (`.github/workflows/gate.yml`)

### 7.1 Matrix cells

| Cell | Runner | Build target | Runtime |
|---|---|---|---|
| A | `ubuntu-24.04` (x86-64) | x86_64-linux | native |
| B | `ubuntu-24.04-arm` (arm64) | aarch64-linux | native |
| C | `macos-15` (Apple silicon) | aarch64-darwin | native |
| D | `ubuntu-24.04` | wasm32-wasip1 | wasmtime (pinned) |
| E | `ubuntu-24.04-arm` | wasm32-wasip1 | wasmtime (pinned) |
| F | `ubuntu-24.04` | wasm32-unknown-unknown | Node (pinned) / V8 |
| G | `ubuntu-24.04-arm` | wasm32-unknown-unknown | Node (pinned) / V8 |

If the repo ends up private and Linux ARM runners aren't on the plan (see Q3/§11), cells B/E/G fall back to `macos-15` for ARM-native coverage and the gap is recorded in `DETERMINISM.md`.

### 7.2 Job graph

`lint` (clippy with covenant config, `-D warnings` on covenant crates) → 7 `cell-*` jobs (each: build, run with 3 in-process repeats, upload `hashes.json` + transcripts) → `fan-in` (download all, `compare.py`: all cells identical **and** equal to `goldens/v0.json`) → `red-path` jobs (§8, wired inverted) — all required. Triggers: every PR, every push to main, and a **weekly cron** (catches runner-image, toolchain-servicing, and runtime drift while the repo is quiet — the gate must decay loudly, not silently).

### 7.3 Metadata

Every `hashes.json` records rustc version, target triple, runtime + version, host arch, timestamp. Not hashed — recorded. When something drifts in 2027, this is the forensic trail.

### 7.4 Guard script

A `fan-in` step fails the PR if `git diff` touches `goldens/**` while `GENERATOR_VERSION` is unchanged, or touches `goldens/**` outside a mint-ceremony PR. Belt (CODEOWNERS) and suspenders (automation).

### 7.5 Golden mint ceremony (`mint-goldens.yml`)

`workflow_dispatch` only. Runs **cell A only**, regenerates `goldens/v0.json` with full provenance (`minted_from: cell A`, toolchain, date, commit), opens a PR. That PR must (a) bump `GENERATOR_VERSION` if any transcript byte changed for existing seeds, (b) pass the full gate — all seven cells must agree with the *new* goldens, and (c) be approved by Daniel (CODEOWNERS enforces). Normal CI has no write path to `goldens/`. Document the ceremony in `DETERMINISM.md`, including the standing rule: **a golden change without a version bump is definitionally an incident, not a fix.**

---

## 8. Red-path fixtures — proving the gate can fail

A gate that has never been red is untested, and this gate is the acceptance boundary agents will be graded by — so its detection claims must be demonstrated, not assumed. Each fixture is a small variant crate/feature under `fixtures/`, built into an alternate harness, run in CI as an **inverted job**: the job's script asserts the gate *detects* the violation (cells disagree, or golden mismatch, or lint/assert fires) and **fails CI if the fixture passes clean**.

- **F1 — Platform libm leak.** W2 with `f64::sin`/`f64::cos` (std inherent methods) instead of `libm::`. Expected: cell C (Apple libm) and/or WASM cells diverge from Linux-native → fan-in red. *This also empirically answers whether std-vs-libm divergence shows on our exact fleet; record the observed divergence pattern in `DETERMINISM.md`.* If — unexpectedly — all cells agree, that is a **finding to report** (the fixture then proves nothing and needs a sharper trigger), not a pass.
- **F2 — Iteration-order entropy.** W3 collects features into a `std::collections::HashMap` and serializes in iteration order. Expected: the 3-repeat in-process self-check fails on the very first cell (per-process `RandomState`). Also expected: the covenant lint refuses to compile this in a kernel crate — which is why fixtures live outside the covenant scope, and why F5 exists.
- **F3 — Silent drift.** One constant in W1 changed (simulates an unreviewed algorithm edit). Expected: all cells agree with each other but mismatch the committed golden → fan-in red. Proves the golden comparison bites independently of cross-cell comparison.
- **F4 — Domain escape.** Inject a NaN-producing input (e.g., `libm::pow(-1.0, 0.5)`) into a W2 variant. Expected: the R6 finiteness assert panics → nonzero exit → cell red.
- **F5 — Lint enforcement.** A file in a covenant-scoped crate using `f64::sin`, `HashMap`, unannotated `+` on `u64`, and `Instant::now()`. Expected: `cargo clippy` exits nonzero on each (may be one file with all four, asserted individually). This is the executable proof for Q1.

Deliverable: each fixture's CI job links a run demonstrating red, referenced from `DETERMINISM.md` ("what this gate has been shown to catch, and when").

---

## 9. Lint and compile-time enforcement

`clippy.toml` (covenant crates; harness/fixtures excluded by crate-level config):

- `disallowed-methods`: all inherent f64 transcendentals (`sin, cos, tan, asin, acos, atan, atan2, sinh, cosh, tanh, exp, exp2, exp_m1, ln, ln_1p, log, log2, log10, powf, powi, min, max, rem_euclid`), `std::time::Instant::now`, `std::time::SystemTime::now`, `std::thread::spawn`, `rand`-anything (shouldn't resolve anyway — no dep).
- `disallowed-types`: `std::collections::HashMap`, `std::collections::HashSet`, `f32` if expressible (see Q1).
- Deny at crate level: `clippy::arithmetic_side_effects` (forces `wrapping_*`/`checked_*` — this is the debug/release parity rule R5 made mechanical).

**The exact clippy path syntax for primitive inherent methods must be verified, not assumed** (Q1) — F5 is the proof it fires. If clippy genuinely cannot express one of these bans, the backstop is a CI textual scan (`grep`-class) over covenant crate sources for the banned method names, documented as such in `DETERMINISM.md`. The hash gate remains the final backstop for anything lints can't see; lints exist to move failure from "hash mismatch, go diagnose" to "compile error, line 42."

---

## 10. Questions the build must answer (record each answer in `DETERMINISM.md` or the PR)

- **Q1 — Clippy syntax for primitive method/type bans.** Does current clippy's `disallowed-methods`/`disallowed-types` resolve paths like `f64::sin` and bare `f32`? Answer empirically via F5. If partial: document exactly which rules fall back to the textual scan.
- **Q2 — `wasm32-wasip1` vs `wasip2`.** Default to `wasip1` (simplest module story under wasmtime). Verify current rustc/wasmtime guidance at build time; if `wasip1` is deprecated or wasmtime's current pin prefers `wasip2`, switch and record. The choice must not affect kernel numerics — if it appears to, stop (OR3).
- **Q3 — ARM runner availability.** Confirm `ubuntu-24.04-arm` is available on the chosen repo's plan (free for public repos; private repos need larger-runner billing). If not: fall back per §7.1 and record the gap.
- **Q4 — Toolchain pin.** Pin the current stable rustc at implementation time in `rust-toolchain.toml`; record the exact version everywhere metadata lives. Same for wasmtime and Node (Node via an exact version in the workflow, not a floating major).
- **Q5 — Debug/release parity.** One-time check: run cell A in debug; hashes must equal release (R5 makes this true by construction — verify anyway). Record the result; thereafter release-only is normative.
- **Q6 — Zero-import instantiation.** Does the `std`-built `wasm32-unknown-unknown` cdylib instantiate with an empty import object? If any imports surface (e.g., panic/abort shims), stub them minimally in `run.mjs`, verify the stubs are unreachable in green runs, and record. If stubbing would alter numerics, stop (OR6).
- **Q7 — sha2 on wasm.** Confirm `sha2` builds warning-free for both wasm targets with default features (its x86 acceleration paths should compile out). If features must be trimmed, pin them explicitly in `gate-runner`.
- **Q8 — mul_add lowering.** W6 probe 3 answers this empirically across all cells; record the observed bit patterns (they should show fused ≠ unfused, both globally consistent).

---

## 11. Inputs needed from Daniel (blocking items flagged)

1. **Repo name + visibility** (blocks Q3 / final matrix shape). Public gets free ARM Linux runners. — *Resolved: standalone public repo `leyline-poc0-determinism-gate`; free ARM runners → full A–G matrix.*
2. **Branch protection**: mark `lint`, all `cell-*`, `fan-in`, and all red-path jobs as required checks; restrict `goldens/**` + `docs/DETERMINISM.md` via CODEOWNERS to Daniel. (Admin click-work — the build will include a `SETUP.md` with the exact checklist.) — *Available on a public repo; configured at T9 via SETUP.md.*
3. **Approve pins** (rustc / wasmtime / Node versions) on the first PR.
4. **License / repo boilerplate** preference, if any. — *Resolved: proprietary, inherited from the leyline repo.*

---

## 12. Acceptance criteria — the proof list

The POC is done when every item below is demonstrated in CI (link the runs):

- **AC1 — Cross-cell equality.** All seven cells (or the documented ARM-fallback set) produce identical digest sets for all 10 seeds, with the 3-repeat in-process self-check passing in every cell, on two consecutive scheduled/manual runs of the full workflow.
- **AC2 — Golden match with provenance.** All cells equal `goldens/v0.json`; the golden file records minted-from cell, toolchain, commit, and date; goldens were produced only via `mint-goldens.yml`.
- **AC3 — Red paths red.** F1–F4 each demonstrably turn the gate red in their inverted CI jobs, with the failure mode matching the expectation stated in §8 (and F1's observed divergence pattern recorded).
- **AC4 — Lints bite.** F5 shows clippy (or the documented textual-scan fallback, per Q1) rejecting each banned construct in covenant scope; covenant crates build clean under `-D warnings` with the full config.
- **AC5 — Golden tamper guard.** A test PR modifying `goldens/v0.json` without a `GENERATOR_VERSION` bump fails the guard step (demonstrated, then closed unmerged).
- **AC6 — Finiteness boundary.** F4's NaN injection produces a panic → nonzero exit → red cell (covered under AC3, called out separately because R6 is the only WASM-spec nondeterminism defense we have).
- **AC7 — Local reproduction.** `make gate` on a dev machine (any arch) reproduces the committed golden digests for the locally runnable subset (native + Node at minimum).
- **AC8 — Docs exist and match reality.** `DETERMINISM.md` contains: the covenant (§4) with per-rule enforcement pointers, the non-promises (§4.5), the mint ceremony (§7.5), answers to Q1–Q8, and links to the AC3/AC4/AC5 demonstration runs. `SETUP.md` contains the §11.2 checklist. This spec is checked in as `docs/POC0-SPEC.md`.
- **AC9 — Required checks configured** (or, if admin access is pending, `SETUP.md` delivered and the configuration verified in review with Daniel).
- **AC10 — Debug/release parity recorded** (Q5).
- **AC11 — Forensic metadata.** Every `hashes.json` artifact carries rustc/target/runtime/host metadata (§7.3), spot-verified in a real run.

---

## 13. Sequencing

| # | Task | Depends on | Est. |
|---|---|---|---|
| T1 | Repo skeleton, workspace, toolchain pin, `DETERMINISM.md` draft (covenant text from §4), this spec checked in | — | 0.5 d |
| T2 | `world-kernel`: SplitMix64 + KATs (with provenance), W1–W6, transcript writer with R6 asserts | T1 | 1 d |
| T3 | `gate-runner` + `gate-cli` native; 3-repeat self-check; `--transcript-out`; local goldens minted informally for dev | T2 | 0.5 d |
| T4 | Covenant lints wired (`clippy.toml`, crate-level denies); F5 fixture proving they fire; Q1 answered | T2 | 0.5 d |
| T5 | wasm32-wasip1 build + wasmtime run; wasm32-unknown-unknown + `gate-wasm` + `run.mjs`; Q2/Q6/Q7 answered | T3 | 0.5–1 d |
| T6 | `gate.yml` matrix + fan-in + `compare.py` with first-divergence reporting; weekly cron; metadata | T5 | 0.5 d |
| T7 | Red fixtures F1–F4 + inverted jobs; record F1 divergence pattern | T6 | 0.5 d |
| T8 | `mint-goldens.yml` ceremony + guard script + CODEOWNERS; formal golden mint via the ceremony; AC5 tamper demo | T6 | 0.5 d |
| T9 | `SETUP.md`, README, Q4/Q5/Q8 recorded, AC sweep with linked runs, handoff review with Daniel | T7, T8 | 0.5 d |

Order T2→T3 before any WASM work: get native green and self-consistent first, so every later divergence has a fixed reference point.

---

## 14. Failure-mode preemption map ("the phone calls this document exists to prevent")

| Future incident | Preemption in this spec |
|---|---|
| Gate was green for months but had never actually caught anything; first real violation sails through | Red-path fixtures F1–F5 run on every PR, inverted — the gate's detection is itself under test (AC3/AC4) |
| Golden quietly re-minted to make a mismatch go away; user worlds orphaned at next regen | OR1, CODEOWNERS, guard script (AC5), ceremony with mandatory version bump (§7.5) |
| Goldens minted from a laptop with an unpinned toolchain; nobody can reproduce them | Mint ceremony runs only in CI, cell A, with recorded provenance (AC2) |
| "It passed wasmtime" but Chrome disagrees in production | V8 (Node) is a normative, gate-blocking cell (F/G), not a nice-to-have |
| A `std::sin` sneaks into the kernel via a helper; works on Linux, drifts on the iPad a year later | Lint wall (F5-proven) + F1 demonstrating the exact failure class + hash gate backstop |
| Debug builds panic where release wraps; agents "fix" it by testing only release | R5: `wrapping_*` discipline + `arithmetic_side_effects` lint + release `overflow-checks = true` + Q5 parity check |
| Runner image or toolchain servicing drifts while nobody is looking; discovered months later mid-feature | Weekly cron on main (§7.2) + forensic metadata (AC11) |
| Hash mismatch appears and all anyone has is "red" — days lost bisecting | Transcript artifacts + `compare.py` first-divergence reporting are required deliverables (§6.4) |
| Signed-zero / min-max / NaN-payload edge nondeterminism surfaces only at scale | R4 bans the unspecified ops; R6 outlaws NaN at the boundary; W6 probes hash the edges on every run |

---

## 15. Definition of done

All acceptance criteria AC1–AC11 demonstrated with linked CI runs; Q1–Q8 answered in writing; `DETERMINISM.md`, `SETUP.md`, and this spec in-repo; goldens minted via the ceremony; Daniel has walked the handoff review (T9) and flipped the required checks. From that moment, the standing rule of the whole project is in force: **no agent-written kernel change merges except through this gate, and no seed is minted for any real world until the gate has been green — and demonstrably capable of red — on main.**

This is the first PoC plan.
