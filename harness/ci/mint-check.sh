#!/usr/bin/env bash
# Re-mint version-bump enforcement (§7.5). If a golden for this generator version
# already exists on the checked-out ref and the freshly-minted seeds differ, then
# GENERATOR_VERSION must bump — a golden change without a version bump is an
# incident, not a fix. First mint of a version passes.
set -euo pipefail
gen="${1:?generator version}"
new="goldens/v${gen}.json"

if git cat-file -e "HEAD:${new}" 2>/dev/null; then
  git show "HEAD:${new}" > /tmp/old-golden.json
  if python3 - "$new" /tmp/old-golden.json <<'PY'
import json, sys
new = json.load(open(sys.argv[1]))["seeds"]
old = json.load(open(sys.argv[2]))["seeds"]
sys.exit(0 if new == old else 1)
PY
  then
    echo "mint-check: re-mint of v${gen} is byte-identical to the committed golden — ok (idempotent)."
  else
    echo "mint-check INCIDENT: v${gen} seed digests changed but GENERATOR_VERSION did not bump (§7.5)."
    echo "Bump world_kernel::GENERATOR_VERSION and re-run the ceremony."
    exit 1
  fi
else
  echo "mint-check: first mint of v${gen} — ok."
fi
