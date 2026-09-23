#!/usr/bin/env python3
"""Per-function self Ir, inclusive Ir and call count, program minus control, per iteration.

usage: fndiff.py PROGRAM_CG CONTROL_CG [N] [MIN]
Needs --compress-strings=no. Self is every cost line not following calls=;
inclusive is self plus the costs on the lines following calls= (callgrind's own
arcs; recursive functions are over-counted, as callgrind_annotate does).
Prints functions whose |self| or |calls| per iteration exceeds MIN (default 0.5),
sorted by self, then the total self difference, which must equal the summary
difference.
"""
import sys


def load(path):
    selfc, incl, calls, obj = {}, {}, {}, {}
    ob = fn = cfn = None
    summary = None
    pending = None
    for line in open(path):
        if line.startswith("summary:"):
            summary = int(line.split()[1])
            continue
        if line.startswith("ob="):
            ob = line[3:].strip()
            continue
        if line.startswith("fn="):
            fn = line[3:].strip()
            obj[fn] = ob
            continue
        if line.startswith("cfn="):
            cfn = line[4:].strip()
            continue
        if line.startswith("calls="):
            pending = int(line[6:].split()[0])
            continue
        if line[:1].isdigit() or line[:1] in "+-*":
            parts = line.split()
            if len(parts) < 2:
                continue
            ir = int(parts[-1])
            if pending is not None:
                incl[fn] = incl.get(fn, 0) + ir
                calls[cfn] = calls.get(cfn, 0) + pending
                pending = None
            else:
                selfc[fn] = selfc.get(fn, 0) + ir
                incl[fn] = incl.get(fn, 0) + ir
    assert sum(selfc.values()) == summary
    return selfc, incl, calls, obj, summary


p = load(sys.argv[1])
c = load(sys.argv[2])
N = int(sys.argv[3]) if len(sys.argv) > 3 else 200000
MIN = float(sys.argv[4]) if len(sys.argv) > 4 else 0.5
names = set(p[0]) | set(c[0]) | set(p[2]) | set(c[2])
rows = []
for f in names:
    s = (p[0].get(f, 0) - c[0].get(f, 0)) / N
    i = (p[1].get(f, 0) - c[1].get(f, 0)) / N
    k = (p[2].get(f, 0) - c[2].get(f, 0)) / N
    if abs(s) >= MIN or abs(k) >= MIN:
        o = (p[3].get(f) or c[3].get(f) or "?").rsplit("/", 1)[-1]
        rows.append((s, i, k, o, f))
rows.sort(key=lambda r: -abs(r[0]))
print("self/it\tincl/it\tcalls/it\tobject\tfunction")
for s, i, k, o, f in rows:
    print(f"{s:.2f}\t{i:.2f}\t{k:.3f}\t{o}\t{f[:150]}")
tot = sum((p[0].get(f, 0) - c[0].get(f, 0)) for f in names) / N
print(f"TOTAL self diff/it {tot:.2f}  summary diff/it {(p[4] - c[4]) / N:.2f}")
