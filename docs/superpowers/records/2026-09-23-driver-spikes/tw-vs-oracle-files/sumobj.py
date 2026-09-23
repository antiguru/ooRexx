#!/usr/bin/env python3
"""Summarise one callgrind file: summary, libc/ld-linux self Ir, ex-libc, and
the self Ir of every function whose name contains "cold_".

usage: sumobj.py CALLGRIND_OUT
prints: summary<TAB>libc<TAB>ld<TAB>exlibc<TAB>cold_fns
Needs --compress-strings=no. Call lines' inclusive costs are skipped.
"""
import re
import sys

ob = fn = None
by_ob = {}
cold = {}
summary = None
skip = False
for line in open(sys.argv[1]):
    if line.startswith("summary:"):
        summary = int(line.split()[1])
        continue
    if line.startswith("ob="):
        ob = line[3:].strip()
        continue
    if line.startswith("fn="):
        fn = line[3:].strip()
        continue
    if line.startswith("calls="):
        skip = True
        continue
    m = re.match(r"^(?:[+-]?\d+|\*|0x[0-9a-f]+)(?: \S+)* (\d+)$", line.strip())
    if line[:1].isdigit() or line[:1] in "+-*":
        parts = line.split()
        if len(parts) < 2:
            continue
        if skip:
            skip = False
            continue
        ir = int(parts[-1])
        by_ob[ob] = by_ob.get(ob, 0) + ir
        if fn and "cold_" in fn:
            cold[fn] = cold.get(fn, 0) + ir
libc = sum(v for k, v in by_ob.items() if k and k.endswith("libc.so.6"))
ld = sum(v for k, v in by_ob.items() if k and "ld-linux" in k)
total = sum(by_ob.values())
assert total == summary, (total, summary)
coldtxt = ",".join(f"{k.split('::')[-1]}={v}" for k, v in cold.items()) or "-"
print(f"{summary}\t{libc}\t{ld}\t{summary - libc - ld}\t{coldtxt}")
