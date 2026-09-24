"""Comment lines that mention a moved unit's name together with a positional
word -- the prose a move can make false. Scans the remaining dispatch.rs and
the new child (for mentions of units that stayed behind).

usage: positional.py PRE_DISPATCH POST_SRC_DIR CHILD
"""
import re, sys
import splitlib
pre, post_dir, child = sys.argv[1:4]
pre_keys = {u["key"] for u in splitlib.load_units(pre)}
child_units = splitlib.load_units(f"{post_dir}/dispatch/{child}.rs")
moved = {u["key"].split("::")[-1].split(" ")[-1] for u in child_units if u["key"] in pre_keys}
stayed = {k.split("::")[-1].split(" ")[-1] for k in pre_keys} - moved
WORDS = re.compile(r"\b(above|below|earlier|later in this|this file|this module|next to|beside)\b", re.I)
for rel, names in (("dispatch.rs", moved), (f"dispatch/{child}.rs", stayed)):
    for n, line in enumerate(open(f"{post_dir}/{rel}"), 1):
        s = line.strip()
        if not s.startswith("//") or not WORDS.search(s):
            continue
        hits = [x for x in names if re.search(r"\b" + re.escape(x) + r"\b", s)]
        if hits:
            print(f"{rel}:{n}: {hits}: {s}")
print("done")
