#!/bin/bash
# usage: unsafe_count.sh PRE_SRC POST_SRC REMOVED_JSON PARENT_RS DESTS(comma)
# rexx-core/tests/unsafe_sites.rs's two predicates, applied line by line to
# code with any `//` comment cut off: `allow(unsafe_code)` or
# `expect(unsafe_code)` (an opt-in), and `unsafe {`, `unsafe fn`,
# `unsafe impl` or `unsafe trait` (a use). Counted over (1) the lines the move
# took out of PRE's parent (REMOVED_JSON) and (2) every destination file.
python3 - "$@" <<'PY'
import json, sys
pre, post, removed, parent, dests = sys.argv[1:6]
NEEDLES = ["allow(unsafe_code)", "expect(unsafe_code)", "unsafe {", "unsafe fn", "unsafe impl", "unsafe trait"]
def hits(lines):
    return [(i, l) for i, l in lines if any(n in l.split("//", 1)[0] for n in NEEDLES)]
lines = open(f"{pre}/{parent}").read().split("\n")
moved = [(i + 1, lines[i]) for i in json.load(open(removed))]
h = hits(moved)
print(f"moved lines of {parent}: {len(moved)} lines, {len(h)} matching the unsafe_sites.rs predicates")
for i, l in h: print(f"  pre {parent}:{i}: {l}")
bad = len(h)
for d in dests.split(","):
    dl = list(enumerate(open(f"{post}/{d}").read().split("\n"), 1))
    h = hits(dl)
    print(f"destination {d}: {len(dl)} lines, {len(h)} matching")
    for i, l in h: print(f"  {d}:{i}: {l}")
    bad += len(h)
print("unsafe: " + ("NONE" if bad == 0 else f"FOUND {bad}"))
PY
