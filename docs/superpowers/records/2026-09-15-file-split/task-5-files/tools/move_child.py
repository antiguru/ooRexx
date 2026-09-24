"""One commit's move: take a child's units (and table rows) out of the
current `dispatch.rs` and write `dispatch/<child>.rs`.

usage: move_child.py CHILD SRC_DIR OUT_REMOVED_JSON "MODULE DOC LINE" ["ROWS DOC"]

Units are taken verbatim. An `impl` block every one of whose members moves is
taken whole, header and closing brace included. The child file gets the
license header, a one-line module doc, a `use super::*;` placeholder the
author replaces by the explicit import list the compiler asks for, the rows
(if any) as the child's own `NATIVE_METHODS`, and the units in source order.
"""
import json
import os
import sys

import splitlib

child, src_dir, removed_json, module_doc = sys.argv[1:5]
rows_doc = sys.argv[5] if len(sys.argv) > 5 else None
plan = json.load(open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "plan.json")))[child]
path = os.path.join(src_dir, "dispatch.rs")
lines = open(path).read().split("\n")[:-1]
units = splitlib.load_units(path)
by_key = {u["key"]: u for u in units}
wanted = set(plan["keys"])

# Group impl members by the block they sit in.
blocks = {}
for u in units:
    if u["key"].startswith("impl ") and "::" in u["key"]:
        # the enclosing block starts at the nearest `impl ` line above
        at = u["first"] - 1
        while not lines[at].startswith("impl "):
            at -= 1
        blocks.setdefault(at, []).append(u["key"])
keys = []
consumed = set()
for u in units:
    key = u["key"]
    if key not in wanted or key in consumed:
        continue
    if key.startswith("impl "):
        start = next(a for a, ms in blocks.items() if key in ms)
        members = blocks[start]
        assert all(m in wanted for m in members), ("partial impl block", start + 1, members)
        end = start
        while lines[end] != "}":
            end += 1
        keys.append(f"LINES:{start + 1}-{end + 1}")
        consumed.update(members)
    else:
        keys.append(key)
missing = wanted - consumed - set(k for k in keys)
assert not missing, missing

new_lines, texts, removed_units = splitlib.take(path, keys + plan["rows"])
LICENSE = lines[:10]
out = LICENSE + ["", f"//! {module_doc}", "", "use super::*;", ""]
if plan["rows"]:
    out += [rows_doc, "pub(super) static NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &["]
    out += [texts[k] for k in plan["rows"]]
    out += ["];", ""]
body = [texts[k] for k in keys]
out += ["\n\n".join(body)]
child_path = os.path.join(src_dir, "dispatch", f"{child}.rs")
assert not os.path.exists(child_path)
open(child_path, "w").write("\n".join(out) + "\n")
open(path, "w").write("\n".join(new_lines) + "\n")
splitlib.write_json(removed_json, removed_units)
print(f"{child}: {len(keys)} blocks, {len(plan['rows'])} rows, {len(removed_units)} lines removed")
