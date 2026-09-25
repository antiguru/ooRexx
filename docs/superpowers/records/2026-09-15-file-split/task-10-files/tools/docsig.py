"""The warnings of one `cargo doc` log as a sorted list of `message | file`
pairs, the file without its line and column: a move that shifts lines keeps
the signature, and a warning that appears, vanishes, or moves to another file
changes it. Warnings with no `-->` location (the summary line) keep "-".

usage: docsig.py LOG
"""
import re, sys
lines = open(sys.argv[1], errors="replace").read().split("\n")
out = []
for i, l in enumerate(lines):
    if not l.startswith("warning"):
        continue
    loc = "-"
    for m in lines[i + 1 : i + 4]:
        g = re.match(r"\s*--> (\S+?):\d+:\d+$", m)
        if g:
            loc = g.group(1)
            break
    out.append(f"{l} | {loc}")
print("\n".join(sorted(out)))
