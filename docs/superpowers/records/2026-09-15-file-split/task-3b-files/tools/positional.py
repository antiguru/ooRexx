"""Comment blocks (runs of consecutive comment lines) that mention a unit
now in another file together with a positional word -- the prose a move can
make false. Scans the remaining parent and the new child: in the parent,
mentions of units that moved; in the child, mentions of units that stayed.

usage: positional.py PRE_PARENT POST_SRC_DIR CHILD
"""
import os, re, sys
import splitlib
PARENT = os.environ.get("SPLIT_PARENT", "dispatch")
PARENT_RS = PARENT + ".rs"
pre, post_dir, child = sys.argv[1:4]
pre_keys = {u["key"] for u in splitlib.load_units(pre)}
child_units = splitlib.load_units(f"{post_dir}/{PARENT}/{child}.rs")
name = lambda k: k.split("::")[-1].split(" ")[-1]
moved = {name(u["key"]) for u in child_units if u["key"] in pre_keys}
stayed = {name(k) for k in pre_keys} - moved
WORDS = re.compile(r"\b(above|below|earlier|later in this|this file|this module|next to|beside|immediately|following|preceding|further (up|down))\b", re.I)
for rel, names in ((PARENT_RS, moved), (f"{PARENT}/{child}.rs", stayed)):
    lines = open(f"{post_dir}/{rel}").read().split("\n")
    i = 0
    while i < len(lines):
        if not lines[i].strip().startswith("//"):
            i += 1
            continue
        j = i
        while j < len(lines) and lines[j].strip().startswith("//"):
            j += 1
        block = " ".join(l.strip().lstrip("/!").strip() for l in lines[i:j])
        if WORDS.search(block):
            hits = sorted(x for x in names if re.search(r"\b" + re.escape(x) + r"\b", block))
            if hits:
                print(f"{rel}:{i + 1}-{j}: {hits}: {WORDS.findall(block)}")
        i = j
print("done")
