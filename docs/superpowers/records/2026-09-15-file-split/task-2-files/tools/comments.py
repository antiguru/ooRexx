"""Every comment line of BASE's dispatch.rs, as a multiset of stripped text,
against the final dispatch.rs plus the files the task created: nothing may be
lost; what was added or edited is listed.

usage: comments.py BASE_SRC FINAL_SRC NEW_FILE...
"""
import sys
from collections import Counter
import rustlex

def comments(path):
    text = open(path).read()
    out = []
    for kind, a, b in rustlex._scan(text):
        if kind in ("comment", "doc", "block"):
            out.extend(l.strip() for l in text[a:b].split("\n"))
    return out

base = Counter(comments(f"{sys.argv[1]}/dispatch.rs"))
final = Counter()
for rel in ["dispatch.rs"] + sys.argv[3:]:
    final.update(comments(f"{sys.argv[2]}/{rel}"))
lost, added = base - final, final - base
print(f"{sum(base.values())} comment lines in BASE dispatch.rs, {sum(final.values())} after")
print(f"lost ({sum(lost.values())}):")
for l, n in sorted(lost.items()):
    print(f"  x{n} {l}")
print(f"added ({sum(added.values())}):")
for l, n in sorted(added.items()):
    print(f"  x{n} {l}")
