"""Every comment line (plain, doc and block) of a set of files at BASE, as a
multiset of stripped text, against the same set plus the files the task
created, at the final commit: what was lost, and what was added.

usage: comments6.py BASE_SRC FINAL_SRC FILE... -- NEW_FILE...
"""
import sys, os
from collections import Counter
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import rustlex

def comments(path):
    text = open(path).read()
    out = []
    for kind, a, b in rustlex._scan(text):
        if kind in ("comment", "doc", "block"):
            out.extend(l.strip() for l in text[a:b].split("\n"))
    return out

base_src, final_src = sys.argv[1:3]
rest = sys.argv[3:]
i = rest.index("--")
old, new = rest[:i], rest[i + 1:]
base, final = Counter(), Counter()
for f in old:
    base.update(comments(f"{base_src}/{f}"))
for f in old + new:
    final.update(comments(f"{final_src}/{f}"))
lost, added = base - final, final - base
print(f"files: {' '.join(old)} -- new: {' '.join(new)}")
print(f"{sum(base.values())} comment lines at BASE, {sum(final.values())} after")
print(f"lost ({sum(lost.values())}):")
for l, n in sorted(lost.items()):
    print(f"  x{n} {l}")
print(f"added ({sum(added.values())}):")
for l, n in sorted(added.items()):
    print(f"  x{n} {l}")
