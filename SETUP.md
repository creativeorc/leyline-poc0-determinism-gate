# SETUP — making the gate the governance boundary

Everything in this repo runs and is green, but until branch protection is turned
on, a red gate does not *mechanically block* a merge — the standing rule ("no
kernel change merges except through a green gate; no seed minted until the gate
is green and demonstrably red-capable") is enforced by discipline. This is the
one-time admin step (spec §11.2, AC9) that makes it mechanical. It is **free on a
public repo**; it changes your solo workflow (you'll push via PRs, not straight
to `main`), so it's your call to flip.

## 1. Protect `main` with required checks

**UI:** Settings → Branches → Add branch ruleset (or "Add rule") for `main`:

- ✅ Require a pull request before merging
  - ✅ Require review from Code Owners  *(enforces `CODEOWNERS` on `goldens/` and `docs/DETERMINISM.md`)*
- ✅ Require status checks to pass before merging
  - ✅ Require branches to be up to date
  - Add these required checks (job names from `gate.yml`):
    - `lint (fmt + clippy -D warnings)`
    - `golden guard (goldens change only via ceremony)`
    - `cell A (x86_64-unknown-linux-gnu / native)`
    - `cell B (aarch64-unknown-linux-gnu / native)`
    - `cell C (aarch64-apple-darwin / native)`
    - `cell D (wasm32-wasip1 / wasmtime)`
    - `cell E (wasm32-wasip1 / wasmtime)`
    - `cell F (wasm32-unknown-unknown / node)`
    - `cell G (wasm32-unknown-unknown / node)`
    - `cell H (wasm32-unknown-unknown / bun)`
    - `cell I (wasm32-unknown-unknown / bun)`
    - `fan-in (all cells must agree)`
    - `red-path F2/F3/F4 (self-check, drift, R6 panic)`
    - `red-path F1 (platform-libm leak must diverge)`
    - `red-path F5 (lint wall must bite)`
- ✅ Do not allow bypassing the above (include administrators)

**CLI equivalent** (requires the `repo` scope, which the current `gh` login has):

```bash
gh api -X PUT repos/creativeorc/leyline-poc0-determinism-gate/branches/main/protection \
  --input - <<'JSON'
{
  "required_status_checks": {
    "strict": true,
    "checks": [
      {"context": "lint (fmt + clippy -D warnings)"},
      {"context": "golden guard (goldens change only via ceremony)"},
      {"context": "cell A (x86_64-unknown-linux-gnu / native)"},
      {"context": "cell B (aarch64-unknown-linux-gnu / native)"},
      {"context": "cell C (aarch64-apple-darwin / native)"},
      {"context": "cell D (wasm32-wasip1 / wasmtime)"},
      {"context": "cell E (wasm32-wasip1 / wasmtime)"},
      {"context": "cell F (wasm32-unknown-unknown / node)"},
      {"context": "cell G (wasm32-unknown-unknown / node)"},
      {"context": "cell H (wasm32-unknown-unknown / bun)"},
      {"context": "cell I (wasm32-unknown-unknown / bun)"},
      {"context": "fan-in (all cells must agree)"},
      {"context": "red-path F2/F3/F4 (self-check, drift, R6 panic)"},
      {"context": "red-path F1 (platform-libm leak must diverge)"},
      {"context": "red-path F5 (lint wall must bite)"}
    ]
  },
  "required_pull_request_reviews": {"required_approving_review_count": 1, "require_code_owner_reviews": true},
  "enforce_admins": true,
  "restrictions": null
}
JSON
```

> Note: check names must match the job names exactly. If you rename a job in
> `gate.yml`, update the required checks too, or that check silently stops gating.

## 2. Goldens are owned by you

`.github/CODEOWNERS` already routes `goldens/` and `docs/DETERMINISM.md` to
`@creativeorc`. With "Require review from Code Owners" on (step 1), that becomes
enforced. The `golden-guard` job is the belt behind it: it fails any PR that
touches `goldens/` outside a `mint/goldens-*` ceremony branch (demonstrated as
AC5).

## 3. The golden mint ceremony (how goldens change)

Never edit `goldens/` by hand. To (re)mint:

1. Actions → **mint-goldens** → Run workflow. It runs cell A, mints
   `goldens/v<gen>.json` with provenance, enforces the version-bump rule, and
   pushes a `mint/goldens-v<gen>` branch.
2. Open a PR from that branch; the gate re-runs and every cell must agree with
   the new golden; the `golden-guard` allows the ceremony branch.
3. A code owner (you) approves and merges. **A golden change without a
   `GENERATOR_VERSION` bump is an incident, not a fix.**

## 4. After flipping protection — verify

Open a trivial no-op PR and confirm all the required checks appear and must pass
before the merge button turns green. That is AC9 satisfied.
