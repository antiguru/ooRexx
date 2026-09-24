"""Comment blocks (runs of consecutive comment lines) that mention a unit
now in another file together with a positional word -- the prose a move can
make false. Scans the parent and every destination: in each, mentions of
units that are now in a different file than the comment.

usage: positional.py PRE_SRC POST_SRC   (SPLIT_PARENT_RS, SPLIT_DESTS)
"""
import os, re, sys
import splitlib
pre, post_dir = sys.argv[1:3]
parent = os.environ["SPLIT_PARENT_RS"]
files = [parent] + os.environ["SPLIT_DESTS"].split(",")
pre_keys = {u["key"] for u in splitlib.load_units(f"{pre}/{parent}")}
name = lambda k: k.split("::")[-1].split(" ")[-1]
where = {}
for rel in files:
    for u in splitlib.load_units(f"{post_dir}/{rel}"):
        if u["key"] in pre_keys:
            where[name(u["key"])] = rel
WORDS = re.compile(r"\b(above|below|earlier|later in this|this file|this module|next to|beside|immediately|following|preceding|further (up|down))\b", re.I)
for rel in files:
    others = {n for n, r in where.items() if r != rel}
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
            hits = sorted(x for x in others if re.search(r"\b" + re.escape(x) + r"\b", block))
            if hits:
                print(f"{rel}:{i + 1}-{j}: {hits}: {WORDS.findall(block)}")
        i = j
print("done")
