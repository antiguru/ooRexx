"""BASE's dispatch.rs against the final dispatch.rs plus every file the task
created, unit by unit: every BASE unit must exist exactly once after, with
the same token stream (modulo visibility and rustfmt's trailing commas) and
the same decoded literals, or be a declared edit. Keys of dispatch/tests.rs
are compared under their `tests::` prefix.

usage: cumulative.py BASE_SRC FINAL_SRC NEW_FILE... 
"""
import sys
from collections import Counter
import os, splitlib
PARENT = os.environ.get("SPLIT_PARENT", "dispatch")
PARENT_RS = os.environ.get("SPLIT_PARENT_RS", PARENT + ".rs")
from splitlib import drop_trailing_commas

base_src, final_src, new = sys.argv[1], sys.argv[2], sys.argv[3:]
DECLARED = set(os.environ.get("SPLIT_DECLARED", "").split("|")) - {""}
base = {u["key"]: u for u in splitlib.load_units(f"{base_src}/{PARENT_RS}")}
after, where = {}, {}
for rel in [PARENT_RS] + new:
    for u in splitlib.load_units(f"{final_src}/{rel}"):
        # A moved test module's items keep their module name as a prefix,
        # as item-tool keys them inline (`tests::`, `object_operand_tests::`).
        k = (os.path.basename(rel)[: -len(".rs")] + "::" if rel.endswith("tests.rs") else "") + u["key"]
        if k.startswith("use ") and rel != PARENT_RS:
            continue
        assert k not in after, k
        after[k] = u; where[k] = rel

def trim(t):
    # The same relaxations instruments.py makes: the trailing comma of a
    # signature's parameter list, and a struct field's widened visibility.
    return splitlib.strip_field_vis(drop_trailing_commas(t.split("\x01")))

bad, declared, vis, per_file = [], [], [], Counter()
for k, u in base.items():
    if k not in after:
        if not k.startswith("use "):
            bad.append(f"MISSING {k}")
        continue
    a = after[k]
    per_file[where[k]] += 1
    same = trim(u["toks"]) == trim(a["toks"]) and u["lits"] == a["lits"]
    if not same:
        (declared if k in DECLARED else bad).append(f"{k} ({where[k]})")
    if u["vis"] != a["vis"]:
        vis.append(f"{k}: {u['vis']} -> {a['vis']} ({where[k]})")
print(f"{len(base)} BASE units; where they are now: {dict(sorted(per_file.items()))}")
print(f"differing and declared: {declared}")
print(f"visibility changes: {len(vis)}")
for v in vis:
    print("  " + v)
print("FAILURES:" if bad else "PASS: every BASE unit is present after, identical or a declared edit")
for b in bad:
    print("  " + b)
sys.exit(1 if bad else 0)
