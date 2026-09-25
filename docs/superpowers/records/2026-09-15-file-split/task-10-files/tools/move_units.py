"""One production move: take named units out of a parent file, verbatim, and
write them, in source order, into a new child file under a given header.

usage: move_units.py PARENT_RS CHILD_RS KEYS_FILE HEADER_FILE OUT_REMOVED_JSON

KEYS_FILE holds one item-tool key per line. The child is the header file's
text, then the units separated by one blank line each. The parent loses each
unit's lines (its span: attributes, doc comments and the plain comments
directly above it) and the blank line before each, as `splitlib.take`
records them. Declarations the parent needs afterwards (`mod`, `use`) are
added by hand and show as INSERTED lines in instrument 1.
"""
import os
import sys

import splitlib

parent, child, keys_file, header_file, removed_json = sys.argv[1:6]
keys = [k for k in open(keys_file).read().split("\n") if k]
order = {u["key"]: u["first"] for u in splitlib.load_units(parent)}
missing = [k for k in keys if k not in order]
assert not missing, missing
keys.sort(key=order.get)
new_lines, texts, removed = splitlib.take(parent, keys)
assert not os.path.exists(child), child
os.makedirs(os.path.dirname(child), exist_ok=True)
header = open(header_file).read()
assert header.endswith("\n")
open(child, "w").write(header + "\n" + "\n\n".join(texts[k] for k in keys) + "\n")
open(parent, "w").write("\n".join(new_lines) + "\n")
splitlib.write_json(removed_json, removed)
print(f"{len(keys)} units, {len(removed)} lines removed from {parent}")
