#!/usr/bin/env python3
"""Self Ir by source line inside functions matching a substring, program minus control, per iteration.

usage: lndiff.py PROGRAM_CG CONTROL_CG FUNC_SUBSTRING [N] [MIN]
Tracks fl=/fi=/fe= so inlined lines are attributed to their own file.
Also prints per-file totals.
"""
import sys


def load(path, sub):
    out = {}
    fn = None
    fl = cur = None
    pos = 0
    pending = False
    for line in open(path):
        if line.startswith("fl="):
            fl = cur = line[3:].strip()
            continue
        if line.startswith(("fi=", "fe=")):
            cur = line[3:].strip()
            continue
        if line.startswith("fn="):
            fn = line[3:].strip()
            cur = fl
            continue
        if line.startswith("calls="):
            pending = True
            continue
        if line[:1].isdigit() or line[:1] in "+-*":
            parts = line.split()
            if len(parts) < 2:
                continue
            t = parts[0]
            if t == "*":
                pass
            elif t[0] in "+-":
                pos += int(t)
            else:
                pos = int(t)
            if pending:
                pending = False
                continue
            if fn and sub in fn:
                k = (cur, pos)
                out[k] = out.get(k, 0) + int(parts[-1])
    return out


sub = sys.argv[3]
N = int(sys.argv[4]) if len(sys.argv) > 4 else 200000
MIN = float(sys.argv[5]) if len(sys.argv) > 5 else 0.5
p = load(sys.argv[1], sub)
c = load(sys.argv[2], sub)
rows = []
files = {}
for k in set(p) | set(c):
    d = (p.get(k, 0) - c.get(k, 0)) / N
    files[k[0]] = files.get(k[0], 0) + d
    if abs(d) >= MIN:
        rows.append((d, k))
rows.sort(key=lambda r: (r[1][0] or "", r[1][1]))
for d, (f, l) in rows:
    short = (f or "?").split("/rust/")[-1].split("/src/")[-1] if f else "?"
    print(f"{d:8.2f}  {short}:{l}")
print("-- per file")
for f, d in sorted(files.items(), key=lambda x: -abs(x[1])):
    if abs(d) >= MIN:
        print(f"{d:8.2f}  {f}")
print(f"TOTAL {sum(files.values()):.2f}")
