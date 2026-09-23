#!/usr/bin/env python3
"""Mean ex-libc per axis and binary from m/results.tsv, deltas against base."""
import collections
import sys

rows = collections.defaultdict(list)
for line in open(sys.argv[1]):
    name, summary, libc, ld, ex, cold = line.rstrip("\n").split("\t")
    rnd, axis, binary = name.split(".")
    rows[(axis, binary)].append(int(ex))
axes = ["rexxcps", "varlookup", "arith", "emptyloop", "dispatch", "compound"]
bins = ["full", "fullni", "half"]
print("| axis | base ex-libc | " + " | ".join(f"{b} ex-libc | {b} delta" for b in bins) + " | spread (max within one build) |")
print("|---|---:|" + "---:|---:|" * len(bins) + "---:|")
for a in axes:
    base = rows[(a, "base")]
    mb = sum(base) / len(base)
    cells = []
    spread = 0.0
    for b in ["base"] + bins:
        v = rows[(a, b)]
        spread = max(spread, (max(v) - min(v)) / min(v) * 100)
    for b in bins:
        v = rows[(a, b)]
        m = sum(v) / len(v)
        cells.append(f"{m:,.0f} | {(m - mb) / mb * 100:+.3f}%")
    print(f"| `{a}` | {mb:,.0f} | " + " | ".join(cells) + f" | {spread:.4f}% |")
