"""A child split that carries a chained native-method slice: units of the
parent (items verbatim, comments above them included) and rows of the
parent's native table go into a new child file.

usage: move_child_rows.py PARENT_RS CHILD_RS OUT_REMOVED_JSON SPEC_JSON

SPEC_JSON: {"doc": [module doc lines], "uses": [use lines],
            "items": [item-tool keys or LINES:a-b, in the order wanted],
            "rows": [row keys, in the order wanted],
            "table_doc": [doc lines of the new table],
            "table_header": "<the header line>",
            "widen": [item keys made `pub(super)`]}
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
def widen(key, text):
    """`pub(super) ` before the item keyword on its declaration line, the
    one edit made to moved text, for an item the parent still reaches."""
    ls = text.split("\n")
    for i, l in enumerate(ls):
        if re.match(r"^(fn|const|static|struct|enum|type) ", l):
            ls[i] = "pub(super) " + l
            return "\n".join(ls)
    raise SystemExit(f"no declaration line to widen in {key}")


for k in spec.get("widen", []):
    assert k in spec["items"], k
    texts[k] = widen(k, texts[k])
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
