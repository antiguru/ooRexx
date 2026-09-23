#!/usr/bin/env python3
"""usage: table.py summary.tsv BASE HEAD -- ex-libc per axis, both rounds, delta %."""
import sys
rows = {}
for line in open(sys.argv[1]):
    b, a, r, summ, libc, ld, ex, cold, rc = line.rstrip("\n").split("\t")
    assert rc == "rc=0", line
    rows.setdefault((b, a), []).append(int(ex))
base, head = sys.argv[2], sys.argv[3]
axes = []
for (b, a) in rows:
    if a not in axes:
        axes.append(a)
print(f"axis\t{base}\t{head}\tdelta%\tspread_base\tspread_head")
for a in axes:
    x = rows[(base, a)]
    y = rows[(head, a)]
    bx, hy = sum(x) / len(x), sum(y) / len(y)
    print(f"{a}\t{bx:.0f}\t{hy:.0f}\t{100 * (hy - bx) / bx:+.3f}\t{max(x) - min(x)}\t{max(y) - min(y)}")
