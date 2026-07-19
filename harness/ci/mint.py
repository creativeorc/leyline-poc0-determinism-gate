#!/usr/bin/env python3
"""Turn cell A's `gate-cli --json` output into a golden file (§3 format, §7.5
ceremony). Deterministic: seeds sorted, fixed key order, so a byte-identical
re-mint produces a byte-identical file. Reads raw JSON on stdin, writes the
golden on stdout. GENERATOR_VERSION lives inside the hashed transcripts, so the
golden mechanically changes only when the kernel does. Stdlib only.
"""
import datetime
import json
import os
import sys

raw = json.load(sys.stdin)
golden = {
    "generator_version": raw["generator_version"],
    "minted_from": "cell A (ubuntu-24.04, x86_64-unknown-linux-gnu, native)",
    "toolchain": raw.get("toolchain", "unknown"),
    "date": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "commit": os.environ.get("GITHUB_SHA", "unknown"),
    "seeds": dict(sorted(raw["seeds"].items())),
}
json.dump(golden, sys.stdout, indent=2)
sys.stdout.write("\n")
