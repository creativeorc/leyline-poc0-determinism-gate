#!/usr/bin/env bash
# F5 inverted check (§8, AC4): the covenant lints MUST reject this fixture.
# Exits 0 only if clippy FAILS *and* every expected violation is reported.
# Fails (exit 1) if clippy passes clean or any violation slips through — that
# would mean the gate's compile-time wall has a hole.
set -uo pipefail
cd "$(dirname "$0")"

out="$(cargo clippy --quiet -- -D warnings 2>&1)"
status=$?

if [ "$status" -eq 0 ]; then
  echo "F5 FAIL: clippy passed, but the covenant lints must reject this fixture."
  echo "$out"
  exit 1
fi

fail=0
check() { # <label> <expected substring of clippy output>
  if grep -qF "$2" <<<"$out"; then
    echo "  ok: $1"
  else
    echo "  MISSING: $1  (expected output containing: $2)"
    fail=1
  fi
}

echo "F5: clippy rejected the fixture (exit $status). Verifying each violation:"
check 'R2 libm (f64::sin)'    'disallowed method `f64::sin`'
check 'R1 f32'                'disallowed type `f32`'
check 'R8 HashMap'            'disallowed type `std::collections::HashMap`'
check 'R5 integer arithmetic' 'arithmetic operation that can potentially result'
check 'R8 Instant::now'       'disallowed method `std::time::Instant::now`'

if [ "$fail" -ne 0 ]; then
  echo "F5 FAIL: at least one covenant violation was NOT caught."
  echo "$out"
  exit 1
fi
echo "F5 PASS: every covenant violation was caught by clippy (Q1: no textual-scan fallback needed)."
