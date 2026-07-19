#!/usr/bin/env python3
"""Annotate a cell's raw `--json` output with forensic metadata (§7.3): the cell
label (env CELL) and a UTC timestamp. Not hashed — recorded, for the day
something drifts in 2027. Reads raw JSON on stdin, writes annotated JSON on
stdout. Stdlib only.
"""
import datetime
import json
import os
import sys

d = json.load(sys.stdin)
d["cell"] = os.environ.get("CELL", "?")
d["timestamp"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
json.dump(d, sys.stdout)
sys.stdout.write("\n")
