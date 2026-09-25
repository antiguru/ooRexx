"""One production move of this task: take units out of a parent file,
verbatim, and write them in source order into a new child file.

usage: move_items10.py PARENT_RS CHILD_RS REMOVED_JSON HEADER_FILE [--impl=HEAD] KEY...

A KEY is an item-tool key, `BLOCK:<impl head>` (a whole `impl` block,
header and closing brace included; asserted to hold only members that are
all taken: `BLOCK:impl SymbolTable`), or `LINES:a-b` (1-based lines outside
any unit, such as a comment that introduces the units below it). A KEY
prefixed with `+` is widened from private to `pub(super)` on its
declaration line, and one prefixed with `^` to `pub(crate)` (Task 10: the
test crate's root reads items of `table_c/<child>.rs`, two levels down). A
doubled prefix (`++`, `^^`) widens every field of the struct too, each
field line at the struct's field indentation getting the same visibility.
These are the only edits made to moved text.

With `--impl=HEAD` (`impl<'a> Inst<'a> {`), the keys are members of one
`impl` block of the parent, each kept at its member indentation, and the
child holds them inside one block opened by HEAD and closed by `}`.

The child is the parent's license header (its first 10 lines), a blank line,
HEADER_FILE's text (the module doc and the child's imports, written by the
author), a blank line, and the units separated by one blank line each. Each
unit takes its span (attributes, doc comments and the plain comments directly
above it) and the blank line before it (splitlib.take, BLANK_INDENTED set so
an indented member's separator goes too). Declarations the parent needs
afterwards (`mod`, re-exports, narrowed imports) are edited by hand and show
in instrument 1 (a).
"""
import json
import os
import re
import sys

os.environ["BLANK_INDENTED"] = "1"
import splitlib

parent, child, removed_json, header_file = sys.argv[1:5]
rest = sys.argv[5:]
impl_head = None
if rest and rest[0].startswith("--impl="):
    impl_head = rest[0][len("--impl="):]
    rest = rest[1:]
VIS = {"+": "pub(super) ", "^": "pub(crate) "}
widen, fields = {}, set()
for k in rest:
    if k[:1] in VIS:
        widen[k.lstrip("+^")] = VIS[k[0]]
        if k[1:2] == k[0]:
            fields.add(k.lstrip("+^"))
keys = [k.lstrip("+^") for k in rest]
lines = open(parent).read().split("\n")[:-1]
units = splitlib.load_units(parent)
by_key = {u["key"]: u for u in units}

take_keys, order = [], {}
for k in keys:
    if k.startswith("BLOCK:"):
        head = k[len("BLOCK:"):]
        members = [u for u in units if u["key"].startswith(head + "::")]
        assert members, k
        start = members[0]["first"] - 1
        while not lines[start].startswith("impl"):
            start -= 1
        end = start
        while lines[end] != "}":
            end += 1
        inside = [u["key"] for u in units if start < u["first"] - 1 < end]
        assert sorted(inside) == sorted(u["key"] for u in members), (k, inside)
        lk = f"LINES:{start + 1}-{end + 1}"
        take_keys.append(lk)
        order[lk] = start
    elif k.startswith("LINES:"):
        take_keys.append(k)
        order[k] = int(k[len("LINES:"):].split("-")[0]) - 1
    else:
        assert k in by_key, k
        if impl_head:
            assert lines[by_key[k]["first"] - 1].startswith("    "), ("not a member", k)
        take_keys.append(k)
        order[k] = by_key[k]["first"] - 1
take_keys.sort(key=order.get)
new_parent, texts, removed = splitlib.take(parent, take_keys)

blocks = []
for k in take_keys:
    t = texts[k]
    if k in widen:
        tl = t.split("\n")
        a, _ = splitlib.span(lines, by_key[k])
        for i in range(by_key[k]["first"] - 1 - a, len(tl)):
            if re.match(r"^(\s*)(fn|const|static|struct|enum|type) ", tl[i]):
                tl[i] = re.sub(r"^(\s*)", r"\g<1>" + widen[k], tl[i], count=1)
                break
        else:
            raise SystemExit(f"no declaration line to widen in {k}")
        if k in fields:
            assert k.startswith("struct "), k
            n = 0
            for j in range(i + 1, len(tl)):
                if re.match(r"^    [a-z_][a-z_0-9]*: ", tl[j]):
                    tl[j] = "    " + widen[k] + tl[j][4:]
                    n += 1
            assert n > 0, ("no field to widen", k)
        t = "\n".join(tl)
    blocks.append(t)
body = "\n\n".join(blocks)
if impl_head:
    body = impl_head + "\n" + body + "\n}"
header = open(header_file).read()
assert header.endswith("\n")
assert not os.path.exists(child), child
os.makedirs(os.path.dirname(child), exist_ok=True)
open(child, "w").write("\n".join(lines[:10]) + "\n\n" + header + "\n" + body + "\n")
open(parent, "w").write("\n".join(new_parent) + "\n")
splitlib.write_json(removed_json, removed)
print(f"{child}: {len(take_keys)} units ({len(widen)} widened), {len(removed)} parent lines removed")
