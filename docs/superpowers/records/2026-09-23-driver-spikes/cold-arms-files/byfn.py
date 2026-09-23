#!/usr/bin/env python3
"""Self Ir per host symbol (callgrind fn=), difference of two files.

usage: byfn.py BASE_OUT HEAD_OUT  -> rows with |delta| > 1000, largest first
"""
import sys


def load(path):
    fn = None
    skip = False
    out = {}
    for line in open(path):
        if line.startswith("fn="):
            fn = line[3:].strip()
            continue
        if line.startswith("calls="):
            skip = True
            continue
        if line[:1].isdigit() or line[:1] in "+-*":
            parts = line.split()
            if len(parts) < 2:
                continue
            if skip:
                skip = False
                continue
            out[fn] = out.get(fn, 0) + int(parts[-1])
    return out


a, b = load(sys.argv[1]), load(sys.argv[2])
rows = [(b.get(k, 0) - a.get(k, 0), k) for k in set(a) | set(b)]
for d, k in sorted(rows, key=lambda r: -abs(r[0])):
    if abs(d) > 1000:
        print(f"{d:+,}\t{a.get(k, 0):,}\t{b.get(k, 0):,}\t{k[:110]}")
