#!/usr/bin/env python3
"""usage: fncmp.py A_CG B_CG [N] -- per-function self Ir, B - A, largest |delta| first (needs --compress-strings=no)."""
import sys
from collections import defaultdict
def load(p):
    out = defaultdict(int); fn = None; skip = False
    for line in open(p):
        if line.startswith("fn="):
            fn = line[3:].strip(); continue
        if line.startswith("calls="):
            skip = True; continue
        if line[:1].isdigit() or line[:1] in "+-*":
            if skip:
                skip = False; continue
            parts = line.split()
            if len(parts) >= 2:
                out[fn] += int(parts[-1])
    return out
a, b = load(sys.argv[1]), load(sys.argv[2])
n = int(sys.argv[3]) if len(sys.argv) > 3 else 15
rows = sorted(set(a) | set(b), key=lambda k: -abs(b.get(k, 0) - a.get(k, 0)))
for k in rows[:n]:
    print(f"{b.get(k,0)-a.get(k,0):+14d}\t{a.get(k,0):14d}\t{b.get(k,0):14d}\t{k[:110]}")
