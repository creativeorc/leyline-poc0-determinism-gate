#!/usr/bin/env python3
"""Fan-in comparator (§6.4) — the reason a red gate says "cell F, seed 0xDEAD…,
section W2TR, byte 1832" instead of just "red".

Two modes (T6):
  compare.py hashes  <golden.json> <cell1.json> <cell2.json> ...
      -> pairwise cell agreement + equality with the golden; nonzero on any
         disagreement, naming the exact (cell, seed) pairs that differ.
  compare.py transcript <a.bin> <b.bin>
      -> first divergent section tag and byte offset.

T1 STUB. Implemented in T6 alongside gate.yml.
"""
import sys

# TODO(T6): implement `hashes` and `transcript` subcommands with first-
# divergence reporting; used by the fan-in job and printed on any red run.

if __name__ == "__main__":
    print("compare.py is a T1 stub; implement in T6 (see docs/POC0-SPEC.md §6.4).",
          file=sys.stderr)
    sys.exit(2)
