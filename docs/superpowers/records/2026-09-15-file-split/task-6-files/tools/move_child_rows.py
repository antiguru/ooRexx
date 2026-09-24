"""A child split that carries a chained native-method slice: units of the
parent (items verbatim, comments above them included) and rows of the
parent's native table go into a new child file.

usage: move_child_rows.py PARENT_RS CHILD_RS OUT_REMOVED_JSON SPEC_JSON

SPEC_JSON: {"doc": [module doc lines], "uses": [use lines],
            "items": [item-tool keys or LINES:a-b, in the order wanted],
            "rows": [row keys, in the order wanted],
            "table_doc": [doc lines of the new table],
            "table_header": "<the header line>"}
The child is: the parent's license header, the module doc, the `use`
lines, the items (blank line between each), then the table: its doc, the
header, the rows verbatim (each keeping its own indentation and the
comments above it), and `];`. The header is checked to equal the parent
table's header apart from the name and visibility, token for token.
"""
import json, os, re, sys
import splitlib

parent, child, removed_json, spec_path = sys.argv[1:5]
spec = json.load(open(spec_path))
lines = open(parent).read().split("\n")[:-1]
keys = spec["items"] + spec["rows"]
new_parent, texts, removed = splitlib.take(parent, keys)
out = lines[:10] + [""] + ["//! " + l if l else "//!" for l in spec["doc"]] + [""]
out += spec["uses"] + [""]
out.append("\n\n".join(texts[k] for k in spec["items"]))
out.append("")
out += spec["table_doc"]
out.append(spec["table_header"])
out += [texts[k] for k in spec["rows"]]
out.append("];")
assert not os.path.exists(child), child
os.makedirs(os.path.dirname(child), exist_ok=True)
open(child, "w").write("\n".join(out) + "\n")
open(parent, "w").write("\n".join(new_parent) + "\n")
splitlib.write_json(removed_json, removed)
print(f"{child}: {len(spec['items'])} items, {len(spec['rows'])} rows, {len(removed)} parent lines removed")
