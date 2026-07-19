#!/usr/bin/env bash
# Golden guard (§7.4). goldens/** may change ONLY via the mint ceremony. On any
# normal PR, a change to goldens/** is RED (OR1) — this is the mechanical belt;
# CODEOWNERS (once branch protection is on) is the suspenders. A golden change
# without a GENERATOR_VERSION bump is definitionally an incident, not a fix.
#
# Usage: golden-guard.sh <base-ref> <head-branch>
set -uo pipefail
base="${1:?base ref (e.g. origin/main)}"
head_branch="${2:-}"

changed="$(git diff --name-only "${base}"...HEAD -- goldens/ 2>/dev/null || true)"

if [ -z "$changed" ]; then
  echo "golden-guard: goldens/ untouched — ok."
  exit 0
fi

echo "golden-guard: goldens/ changed in this PR:"
echo "$changed" | sed 's/^/  /'

case "$head_branch" in
  mint/goldens-*)
    echo "golden-guard: mint-ceremony branch ($head_branch) — allowed (§7.5)."
    exit 0
    ;;
  *)
    echo "GUARD RED: goldens/** changed outside the mint ceremony."
    echo "Goldens change ONLY via mint-goldens.yml with code-owner approval (OR1, §7.5)."
    echo "A golden change without a GENERATOR_VERSION bump is an incident, not a fix."
    exit 1
    ;;
esac
