#!/usr/bin/env python3
"""Fan-in comparator + transcript first-divergence reporter (§6.4).

The reason a red gate says "cell G, seed 0xDEAD…, section W2TR, byte 1832"
instead of just "red". Stdlib only; no dependencies.

Subcommands:
  compare.py hashes [--golden <v0.json>] <hashes1.json> <hashes2.json> ...
      Every cell must agree with every other cell on every seed, and — if a
      golden is given and exists — with the golden. Exits nonzero on any
      disagreement or missing seed, naming exactly which (cell, seed) differ.

  compare.py transcript <a.bin> <b.bin>
      Report the first divergent byte, and which transcript section (§5.6) it
      falls in, with the offset inside that section.
"""

import argparse
import json
import sys


# ---------------------------------------------------------------------------
# hashes mode
# ---------------------------------------------------------------------------
def _load(path):
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def _label(path, d):
    parts = [d.get("cell"), d.get("target"), d.get("runtime"), d.get("host_arch")]
    joined = "/".join(str(p) for p in parts if p)
    return joined or path


def cmd_hashes(args):
    sources = []  # (label, seeds_dict)
    for path in args.cells:
        d = _load(path)
        sources.append((_label(path, d), d.get("seeds", {})))

    if args.golden:
        try:
            g = _load(args.golden)
            sources.append(("GOLDEN", g.get("seeds", {})))
        except FileNotFoundError:
            print(f"note: golden {args.golden} not found — cross-cell comparison only "
                  "(goldens are minted via the ceremony, T8).")

    if len(sources) < 2:
        print("note: fewer than 2 sources; nothing to cross-check.")
        return 0

    all_labels = [lab for lab, _ in sources]
    per_seed = {}  # seed -> {label: digest}
    for lab, seeds in sources:
        for seed, dig in seeds.items():
            per_seed.setdefault(seed, {})[lab] = dig

    mismatches = []
    for seed in sorted(per_seed):
        got = per_seed[seed]
        missing = [lab for lab in all_labels if lab not in got]
        distinct = set(got.values())
        if missing or len(distinct) != 1:
            mismatches.append((seed, got, missing))

    nseeds = len(per_seed)
    if not mismatches:
        extra = " + golden" if "GOLDEN" in all_labels else ""
        print(f"OK: {len(sources)} sources{extra} agree on all {nseeds} seeds.")
        return 0

    print(f"RED: {len(mismatches)} seed(s) disagree across {len(sources)} sources:")
    for seed, got, missing in mismatches:
        print(f"  seed {seed}:")
        by_dig = {}
        for lab, dig in got.items():
            by_dig.setdefault(dig, []).append(lab)
        for dig, labs in sorted(by_dig.items()):
            print(f"    {dig}  <- {', '.join(sorted(labs))}")
        if missing:
            print(f"    (MISSING from: {', '.join(sorted(missing))})")
    return 1


# ---------------------------------------------------------------------------
# transcript mode
# ---------------------------------------------------------------------------
HEADER_LEN = 16  # magic(4) + generator_version(4) + seed(8)


def _sections(data):
    """Yield (tag, header_start, payload_start, payload_end) per §5.6 framing."""
    off = HEADER_LEN
    n = len(data)
    while off + 12 <= n:
        tag = data[off:off + 4].decode("ascii", "replace")
        length = int.from_bytes(data[off + 4:off + 12], "little")
        payload_start = off + 12
        payload_end = payload_start + length
        yield (tag, off, payload_start, payload_end)
        if payload_end <= off:  # guard against a corrupt/zero-length loop
            break
        off = payload_end


def _locate(data, pos):
    """Human location of byte offset `pos` within a transcript."""
    if pos < 4:
        return ("header:magic", pos)
    if pos < 8:
        return ("header:generator_version", pos - 4)
    if pos < HEADER_LEN:
        return ("header:seed", pos - 8)
    for tag, hstart, pstart, pend in _sections(data):
        if hstart <= pos < pend:
            if pos < pstart:
                return (f"{tag}:framing", pos - hstart)
            return (tag, pos - pstart)
    return ("beyond-last-section", pos)


def cmd_transcript(args):
    with open(args.a, "rb") as f:
        a = f.read()
    with open(args.b, "rb") as f:
        b = f.read()

    if a == b:
        print(f"identical: both {len(a)} bytes.")
        return 0

    m = min(len(a), len(b))
    div = next((i for i in range(m) if a[i] != b[i]), None)
    if div is None:
        sec, soff = _locate(a if len(a) > len(b) else b, m)
        print(f"RED: length differs ({len(a)} vs {len(b)} bytes); "
              f"common prefix {m} bytes ends in section {sec} (offset {soff}).")
        return 1

    sec, soff = _locate(a, div)
    print(f"RED: first divergence at byte {div} — section {sec}, offset {soff}: "
          f"{a[div]:#04x} vs {b[div]:#04x}")
    return 1


def main(argv=None):
    p = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    sub = p.add_subparsers(dest="cmd", required=True)

    h = sub.add_parser("hashes", help="compare cell hashes.json files (+ optional golden)")
    h.add_argument("--golden")
    h.add_argument("cells", nargs="+")
    h.set_defaults(func=cmd_hashes)

    t = sub.add_parser("transcript", help="first-divergence report for two .bin transcripts")
    t.add_argument("a")
    t.add_argument("b")
    t.set_defaults(func=cmd_transcript)

    args = p.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
