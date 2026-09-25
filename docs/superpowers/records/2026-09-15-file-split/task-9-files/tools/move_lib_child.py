"""One commit's move out of a parent file into a new sibling or child file.

usage: move_lib_child.py PARENT_RS CHILD_RS OUT_REMOVED_JSON "MODULE DOC" KEY...

Each KEY is an item-tool key of PARENT_RS (`fn x`, `struct Y`,
`impl Interp::fn z`), or `LINES:a-b` for a whole block such as an `impl`
other than `impl Interp`. Units are taken verbatim, comments above them
included. Moved `impl Interp` members, each still indented as a member, go
into one `impl Interp { ... }` block of the child. The child gets the
parent's license header, a one-line module doc, and a `use super::*;`
placeholder the author replaces by the explicit imports the compiler asks
for; then the moved items that sat above the first moved member, the
members, and the items that sat below, each group in source order.
"""
import json
import os
import sys

os.environ["BLANK_INDENTED"] = "1"
import splitlib

parent, child, removed_json, module_doc = sys.argv[1:5]
keys = sys.argv[5:]
lines = open(parent).read().split("\n")[:-1]
units = {u["key"]: u for u in splitlib.load_units(parent)}
first = {}
for k in keys:
    if k.startswith("LINES:"):
        first[k] = int(k[6:].split("-")[0])
    else:
        assert k in units, k
        first[k] = units[k]["first"]
members = sorted((k for k in keys if k.startswith("impl Interp::")), key=first.get)
items = sorted((k for k in keys if not k.startswith("impl Interp::")), key=first.get)
new_lines, texts, removed = splitlib.take(parent, items + members)
m0 = first[members[0]] if members else 10**9
parts = [texts[k] for k in items if first[k] < m0]
if members:
    parts.append("impl Interp {\n" + "\n\n".join(texts[k] for k in members) + "\n}")
parts += [texts[k] for k in items if first[k] > m0]
out = lines[:10] + ["", f"//! {module_doc}", "", "use super::*;", "", "\n\n".join(parts)]
assert not os.path.exists(child), child
open(child, "w").write("\n".join(out) + "\n")
open(parent, "w").write("\n".join(new_lines) + "\n")
splitlib.write_json(removed_json, removed)
print(f"{os.path.basename(child)}: {len(items)} items, {len(members)} members, {len(removed)} lines removed")
