#!/usr/bin/env bash
# Single-cell red-path checks F2/F3/F4 (§8). Exits 0 only if each fixture
# exhibits its expected red behavior — i.e. the gate's detection mechanism bites.
# (F1 is cross-cell; the `red-path-f1` job in gate.yml compares platforms.)
set -uo pipefail
cd "$(dirname "$0")"

cargo build --quiet --release --bin f2 --bin f3 --bin f4 || { echo "build failed"; exit 1; }
fail=0

echo "== F2: HashMap iteration entropy (3-repeat self-check would go red) =="
out=$(./target/release/f2); rc=$?
if [ "$rc" -eq 0 ] && grep -q "F2 DEMONSTRATED" <<<"$out"; then
  echo "  ok"
else
  echo "  FAIL (rc=$rc)"; echo "$out"; fail=1
fi

echo "== F3: silent drift (golden comparison would go red) =="
out=$(./target/release/f3); rc=$?
if [ "$rc" -eq 0 ] && grep -q "F3 DEMONSTRATED" <<<"$out"; then
  echo "  ok"
else
  echo "  FAIL (rc=$rc)"; echo "$out"; fail=1
fi

echo "== F4: domain escape (R6 assert must panic) =="
err=$(./target/release/f4 2>&1); rc=$?
if [ "$rc" -ne 0 ] && grep -q "R6 violation" <<<"$err"; then
  echo "  ok (rc=$rc, R6 assert fired)"
else
  echo "  FAIL (rc=$rc; R6 assert did not fire)"; echo "$err"; fail=1
fi

if [ "$fail" -eq 0 ]; then
  echo "RED-PATHS F2/F3/F4: all demonstrated the gate goes red."
else
  echo "RED-PATHS: at least one fixture did NOT demonstrate its red-path."
  exit 1
fi
