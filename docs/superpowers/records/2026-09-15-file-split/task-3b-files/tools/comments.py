"""Every comment line of BASE's dispatch.rs, as a multiset of stripped text,
against the final dispatch.rs plus the files the task created: nothing may be
lost; what was added or edited is listed.

usage: comments.py BASE_SRC FINAL_SRC NEW_FILE...
"""
import sys
from collections import Counter
import os, rustlex
PARENT = os.environ.get("SPLIT_PARENT", "dispatch")
PARENT_RS = PARENT + ".rs"

def comments(path):
    text = open(path).read()
    out = []
    for kind, a, b in rustlex._scan(text):
        if kind in ("comment", "doc", "block"):
            out.extend(l.strip() for l in text[a:b].split("\n"))
    return out

base = Counter(comments(f"{sys.argv[1]}/{PARENT_RS}"))
final = Counter()
for rel in [PARENT_RS] + sys.argv[3:]:
    final.update(comments(f"{sys.argv[2]}/{rel}"))
lost, added = base - final, final - base
print(f"{sum(base.values())} comment lines in BASE {PARENT_RS}, {sum(final.values())} after")
print(f"lost ({sum(lost.values())}):")
for l, n in sorted(lost.items()):
    print(f"  x{n} {l}")
print(f"added ({sum(added.values())}):")
for l, n in sorted(added.items()):
    print(f"  x{n} {l}")
