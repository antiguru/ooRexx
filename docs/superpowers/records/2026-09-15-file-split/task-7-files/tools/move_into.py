"""Moves units of a parent file into a file that already exists (or a new
one), verbatim, comments above them included, in their source order.

usage: move_into.py PARENT_RS DEST_RS OUT_REMOVED_JSON PLACE KEY...

PLACE is `end` (appended after the destination's last line, one blank line
between), `after:<item-tool key>` (after that unit of the destination, one
blank line on each side), or `before:<item-tool key>`. A KEY prefixed with
`+` is also widened from private to `pub(super)` on its declaration line
(the only edit made to moved text). A new destination is created with the
parent's license header, the module doc given in NEW_DOC (environment
variable, lines separated by `\\n`) and nothing else before the units.
"""
import json, os, re, sys
import splitlib

parent, dest, removed_json, place = sys.argv[1:5]
raw = sys.argv[5:]
widen = {k[1:] for k in raw if k.startswith("+")}
keys = [k.lstrip("+") for k in raw]
units = {u["key"]: u for u in splitlib.load_units(parent)}
for k in keys:
    assert k in units, k
keys.sort(key=lambda k: units[k]["first"])
new_parent, texts, removed = splitlib.take(parent, keys)
blocks = []
for k in keys:
    t = texts[k]
    if k in widen:
        lines = t.split("\n")
        u = units[k]
        a, _ = splitlib.span(open(parent).read().split("\n")[:-1], u)
        # the declaration line: the first line at or after the unit's first
        # line (attributes/docs skipped) that starts the item
        for i in range(u["first"] - 1 - a, len(lines)):
            if re.match(r"^(\s*)(fn|const|static|struct|enum|type) ", lines[i]):
                lines[i] = re.sub(r"^(\s*)", r"\1pub(super) ", lines[i], count=1)
                break
        else:
            raise SystemExit(f"no declaration line to widen in {k}")
        t = "\n".join(lines)
    blocks.append(t)
body = "\n\n".join(blocks)
if os.path.exists(dest):
    d = open(dest).read().split("\n")[:-1]
    if place == "end":
        d = d + [""] + body.split("\n")
    else:
        how, key = place.split(":", 1)
        du = {u["key"]: u for u in splitlib.load_units(dest)}[key]
        if how == "after":
            at = du["last"]
            d = d[:at] + [""] + body.split("\n") + d[at:]
        else:
            at, _ = splitlib.span(d, du)
            d = d[:at] + body.split("\n") + [""] + d[at:]
else:
    lic = open(parent).read().split("\n")[:10]
    doc = ["//! " + l if l else "//!" for l in os.environ["NEW_DOC"].split("\\n")]
    d = lic + [""] + doc + [""] + body.split("\n")
open(dest, "w").write("\n".join(d) + "\n")
open(parent, "w").write("\n".join(new_parent) + "\n")
splitlib.write_json(removed_json, removed)
print(f"{dest}: {len(keys)} units ({len(widen)} widened), {len(removed)} parent lines removed")
