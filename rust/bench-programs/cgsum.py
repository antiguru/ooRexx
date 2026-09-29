#!/usr/bin/env python3
"""Summarise one callgrind output file by object.

usage: cgsum.py CALLGRIND_OUT
prints: summary<TAB>libc<TAB>ld<TAB>exlibc
where libc and ld are the self instructions attributed to libc.so.6 and to
ld-linux, and exlibc is the summary less both. glibc's allocator varies from
run to run of the same binary (measured 0.57% on arith.rex), which is why the
comparison is taken on exlibc. Needs --compress-strings=no. Call lines'
inclusive costs are skipped so each instruction is counted once.
"""
import sys

ob = None
by_ob = {}
summary = None
skip = False
for line in open(sys.argv[1]):
    if line.startswith("summary:"):
        summary = int(line.split()[1])
        continue
    if line.startswith("ob="):
        ob = line[3:].strip()
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
        by_ob[ob] = by_ob.get(ob, 0) + int(parts[-1])
libc = sum(v for k, v in by_ob.items() if k and k.endswith("libc.so.6"))
ld = sum(v for k, v in by_ob.items() if k and "ld-linux" in k)
assert sum(by_ob.values()) == summary, (sum(by_ob.values()), summary)
print(f"{summary}\t{libc}\t{ld}\t{summary - libc - ld}")
