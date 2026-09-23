#!/usr/bin/env python3
"""Classify one clause's instructions (addrtsv output) by address range into purposes.

usage: classify.py TSV RANGES
RANGES lines: lo_hex hi_hex purpose   (inclusive lo, inclusive hi; first match wins)
Prints purpose totals, the total, and any unclassified rows; also the %rsp-relative count.
"""
import re
import sys

ranges = []
for line in open(sys.argv[2]):
    line = line.strip()
    if not line or line.startswith("#"):
        continue
    lo, hi, purpose = line.split(None, 2)
    ranges.append((int(lo, 16), int(hi, 16), purpose))
tot = {}
order = []
unc = []
stack = 0.0
total = 0.0
for line in open(sys.argv[1]):
    ir, addr, fn, f, ln, asm = line.rstrip("\n").split("\t")
    ir = float(ir)
    a = int(addr, 16)
    total += ir
    if re.search(r"\(%rsp\)|%rsp\)", asm) or asm.startswith(("push", "pop")):
        stack += ir
    for lo, hi, p in ranges:
        if lo <= a <= hi:
            if p not in tot:
                order.append(p)
            tot[p] = tot.get(p, 0) + ir
            break
    else:
        unc.append(line.strip())
for p in order:
    print(f"{tot[p]:7.1f}  {p}")
print(f"{total:7.1f}  TOTAL  (stack-relative or push/pop: {stack:.1f})")
for u in unc:
    print("UNCLASSIFIED", u)
