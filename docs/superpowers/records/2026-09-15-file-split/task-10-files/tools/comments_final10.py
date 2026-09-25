"""Every comment line of a BASE parent is still present after the task: the
multiset of comment lines (rustlex's comment, doc and block-comment tokens,
each line stripped) of BASE's parent against the final parent plus every
file the task created from it. Lists what was lost (must be none) and what
was added.

usage: comments_final10.py BASE_ROOT FINAL_ROOT PARENT_RS NEW_FILE...
"""
import os, sys
from collections import Counter
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import rustlex

base, final, parent = sys.argv[1:4]
new = sys.argv[4:]


def comment_lines(path):
    text = open(path).read()
    out = []
    for kind, a, b in rustlex._scan(text):
        if kind in ("comment", "doc", "block"):
            out += [l.strip() for l in text[a:b].split("\n")]
    return Counter(out)


before = comment_lines(os.path.join(base, parent))
after = Counter()
for rel in [parent] + new:
    after += comment_lines(os.path.join(final, rel))
lost, added = before - after, after - before
print(f"== {parent} -> {' '.join(new)}")
print(f"{sum(before.values())} comment lines in BASE {parent}, {sum(after.values())} after")
print(f"lost ({sum(lost.values())}):")
for l, n in sorted(lost.items()):
    print(f"  x{n} {l}")
print(f"added ({sum(added.values())}):")
for l, n in sorted(added.items()):
    print(f"  x{n} {l}")
sys.exit(1 if lost else 0)
