#!/usr/bin/env python3
"""Per-function differences between two callgrind output files.

usage: cgdiff.py A B   (recorded with --compress-strings=no)
prints: self-instruction differences (B - A), then call-count differences,
one function per line, largest last.
"""
import sys


def load(path):
    fn = cfn = None
    cost, calls = {}, {}
    skip = False
    for line in open(path):
        if line.startswith("fn="):
            fn = line[3:].strip()
        elif line.startswith("cfn="):
            cfn = line[4:].strip()
        elif line.startswith("calls="):
            calls[cfn] = calls.get(cfn, 0) + int(line.split()[0][6:])
            skip = True
        elif line[:1].isdigit() or line[:1] in "+-*":
            parts = line.split()
            if len(parts) < 2:
                continue
            if skip:
                skip = False
                continue
            cost[fn] = cost.get(fn, 0) + int(parts[-1])
    return cost, calls


(ca, na), (cb, nb) = load(sys.argv[1]), load(sys.argv[2])
for title, a, b in (("self", ca, cb), ("calls", na, nb)):
    print(f"# {title}")
    diffs = sorted((b.get(k, 0) - a.get(k, 0), k) for k in set(a) | set(b))
    for d, k in diffs:
        if d:
            print(f"{d:+}\t{k}")
