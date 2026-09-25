"""Cross-reference of a file's units: for each unit, the other units of the
same file whose names its token stream mentions, and the units that mention it.

usage: xref.py FILE [SECTIONS_FILE]
"""
import sys, re
sys.path.insert(0, __import__("os").path.dirname(__file__))
import splitlib
path = sys.argv[1]
units = [u for u in splitlib.load_units(path) if not u["key"].startswith("use ")]
def name(k):
    return k.split("::")[-1].split(" ")[-1]
names = {name(u["key"]): u for u in units if not u["key"].startswith("row ")}
lines = open(path).read().split("\n")
# section of each unit: the last `// ---- x ----` line above it
secs = [(i + 1, l) for i, l in enumerate(lines) if l.startswith("// ----")]
def sec(u):
    s = [l for (n, l) in secs if n < u["first"]]
    return s[-1][8:-5] if s else "-"
uses = {}
for u in units:
    toks = set(u["toks"].split("\x01"))
    uses[u["key"]] = sorted(n for n in names if n in toks and n != name(u["key"]))
used_by = {n: [] for n in names}
for k, us in uses.items():
    for n in us:
        used_by[n].append(k)
for u in units:
    k = u["key"]
    n = name(k)
    print(f"{u['first']}-{u['last']} [{sec(u)}] {k}")
    print(f"    uses: {', '.join(uses[k])}")
    if n in used_by and not k.startswith("row "):
        print(f"    used by: {', '.join(name(x) if not x.startswith('row ') else x for x in used_by[n])}")
