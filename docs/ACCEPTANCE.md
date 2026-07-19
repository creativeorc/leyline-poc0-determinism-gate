# POC 0 — Acceptance sweep (AC1–AC11)

Status of the proof list in `POC0-SPEC.md` §12. "Run" links are GitHub Actions
runs in `creativeorc/leyline-poc0-determinism-gate`.

Key runs:
- **7-cell green** (first): [run 29673296970](https://github.com/creativeorc/leyline-poc0-determinism-gate/actions/runs/29673296970)
- **red-paths F1–F4** demonstrated: [run 29673743586](https://github.com/creativeorc/leyline-poc0-determinism-gate/actions/runs/29673743586)
- **golden anchored on main** (8 sources + golden agree): [run 29674341399](https://github.com/creativeorc/leyline-poc0-determinism-gate/actions/runs/29674341399)

| AC | Requirement | Status | Evidence |
|---|---|---|---|
| **AC1** | Cross-cell equality; 3-repeat in-process self-check per cell; two consecutive full-workflow runs | ✅ | `fan-in` reports `OK: 7 sources agree on all 10 seeds` (and `8 sources + golden` post-mint) across cells A–G, on every push to `main`. `run_all(3)` self-check runs in every cell. |
| **AC2** | Golden match with provenance; goldens minted only via the ceremony | ✅ | `goldens/v0.json` minted from cell A via `mint-goldens.yml` (commit `9955b20`), records `minted_from`/`toolchain`/`date`/`commit`. Main fan-in: `8 sources + golden agree`. |
| **AC3** | F1–F4 turn the gate red; F1 divergence pattern recorded | ✅ | `red-path-f234` + `red-path-f1` (run 29673743586). F1 three-way divergence (glibc/Apple/wasm) recorded in `DETERMINISM.md`. |
| **AC4** | Lints bite (F5); covenant crates clean under `-D warnings` | ✅ | `red-path-f5` green (inverted); `lint` job green. Q1: all bans resolve in clippy — no textual-scan fallback. |
| **AC5** | A PR editing goldens without a version bump fails the guard | ✅ | Tamper PR #2: `golden-guard` failed (`GUARD RED: goldens/** changed outside the mint ceremony`); closed unmerged. |
| **AC6** | Finiteness boundary — NaN → panic → red cell | ✅ | F4 (`red-path-f234`): `sqrt(-1)` → R6 assert panics, nonzero exit. R6 is the only defense against WASM NaN-payload nondeterminism. |
| **AC7** | Local reproduction — `make gate` reproduces the golden digests | ✅ | `make gate` (native + Node + wasmtime) → `OK … agree`; seed0 `5334a21c…` matches the golden and coreutils `sha256sum`. |
| **AC8** | Docs exist and match reality | ✅ | `DETERMINISM.md` (covenant R1–R13, non-promises, mint ceremony, Q1–Q8 answered, red-path evidence + run links), `SETUP.md`, `docs/POC0-SPEC.md`, this file. |
| **AC9** | Required checks configured | ⏳ delivered | `SETUP.md` has the exact branch-protection + required-checks checklist. Flipping it is a one-time admin step (free on this public repo); AC9's documented alternative is satisfied. |
| **AC10** | Debug/release parity recorded (Q5) | ✅ | Q5 in `DETERMINISM.md`: debug and release digests byte-identical on cell A; release-only normative. |
| **AC11** | Forensic metadata on every `hashes.json` | ✅ | Each records `generator_version`, `target`, `runtime`, `toolchain`, `runtime_version`, `host_arch`, `cell`, `timestamp` (`harness/ci/annotate.py`, §7.3). |

## Questions Q1–Q8

All answered in `DETERMINISM.md`:
- **Q1** clippy resolves every ban (no textual-scan fallback).
- **Q2** `wasm32-wasip1` under wasmtime 46.0.1.
- **Q3** free ARM Linux runners (public repo) — full A–G matrix.
- **Q4** pins: rustc 1.93.1, wasmtime 46.0.1, Node 22.11.0.
- **Q5** debug == release (byte-identical).
- **Q6** zero-import wasm instantiation.
- **Q7** `sha2` clean on both wasm targets.
- **Q8** `mul_add` fused `0x3c90000000000000` ≠ unfused `0x0`, both globally consistent.

## Definition of done (§15)

Met, with one deliberate hold: **no seed is minted for any real world until the
gate has been green — and demonstrably red-capable — on `main`.** It now is (7
cells + golden green; F1–F5 + AC5 red-capable). The remaining step to make the
standing rule *mechanical* rather than disciplinary is flipping branch protection
per `SETUP.md` (AC9).
