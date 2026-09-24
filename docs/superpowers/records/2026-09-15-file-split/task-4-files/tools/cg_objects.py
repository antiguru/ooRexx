"""Self instruction cost per ELF object in a callgrind output file, and the
`summary:` total they must add up to.

usage: cg_objects.py CALLGRIND_OUT
prints: summary, then one line per object, then `excluding libc/ld-linux: N`.

Cost lines directly after a `calls=` line are inclusive call costs and are
skipped; `ob=`/`cob=` share one compressed name table.
"""
import re
import sys
from collections import defaultdict

names = {}


def resolve(value):
    m = re.match(r"\((\d+)\)(?: (.*))?$", value)
    if not m:
        return value
    if m.group(2) is not None:
        names[m.group(1)] = m.group(2)
    return names[m.group(1)]


per_object = defaultdict(int)
summary = None
current = None
skip_next = False
for line in open(sys.argv[1]):
    line = line.rstrip("\n")
    if line.startswith("summary:"):
        summary = int(line.split()[1])
        continue
    if line.startswith("ob="):
        current = resolve(line[3:])
        continue
    if line.startswith("cob="):
        resolve(line[4:])
        continue
    if line.startswith("calls="):
        skip_next = True
        continue
    if line and (line[0].isdigit() or line[0] in "+-*"):
        if skip_next:
            skip_next = False
            continue
        parts = line.split()
        if len(parts) >= 2:
            per_object[current] += int(parts[1])
        continue

total = sum(per_object.values())
assert total == summary, (total, summary)
print(f"summary: {summary}")
for ob, cost in sorted(per_object.items(), key=lambda kv: -kv[1]):
    print(f"{cost}\t{ob}")
excluded = sum(c for ob, c in per_object.items() if ob and ("libc.so.6" in ob or "ld-linux" in ob))
print(f"excluding libc/ld-linux: {summary - excluded}")
