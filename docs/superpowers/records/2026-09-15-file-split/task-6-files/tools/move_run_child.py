"""One commit's move: take a child's units out of the current `run.rs` and
write `run/<child>.rs`.

usage: move_run_child.py CHILD SRC_DIR OUT_REMOVED_JSON "MODULE DOC LINE"

Units are taken verbatim. An `impl` block other than `impl Interp` is taken
whole, header and closing brace included, and only when every member moves.
The moved `impl Interp` members, each still indented as a member, go into one
`impl Interp { ... }` block of the child. The child file gets the license
header, a one-line module doc, a `use super::*;` placeholder the author
replaces by the explicit import list the compiler asks for, then the moved
items that sat above `run.rs`'s main `impl Interp` block, the moved members,
and the moved items that sat below it, each group in source order.
"""
import json
import os
import sys

os.environ["BLANK_INDENTED"] = "1"
import splitlib

child, src_dir, removed_json, module_doc = sys.argv[1:5]
plan = json.load(open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "plan-run.json")))[child]
path = os.path.join(src_dir, "run.rs")
lines = open(path).read().split("\n")[:-1]
units = splitlib.load_units(path)
by_key = {u["key"]: u for u in units}
main_impl = lines.index("impl Interp {")
wanted_items = set(plan["items"])
wanted_methods = set(plan["methods"])
assert not (wanted_items - by_key.keys()), wanted_items - by_key.keys()
assert not (wanted_methods - by_key.keys()), wanted_methods - by_key.keys()

blocks = {}
for u in units:
    if u["key"].startswith("impl ") and not u["key"].startswith("impl Interp::"):
        at = u["first"] - 1
        while not lines[at].startswith("impl "):
            at -= 1
        blocks.setdefault(at, []).append(u["key"])
item_keys, method_keys, consumed = [], [], set()
for u in units:
    key = u["key"]
    if key in wanted_methods:
        assert key.startswith("impl Interp::"), key
        method_keys.append(key)
    elif key in wanted_items and key not in consumed:
        if key.startswith("impl "):
            start = next(a for a, ms in blocks.items() if key in ms)
            members = blocks[start]
            assert all(m in wanted_items for m in members), ("partial impl block", start + 1, members)
            end = start
            while lines[end] != "}":
                end += 1
            item_keys.append((start, f"LINES:{start + 1}-{end + 1}"))
            consumed.update(members)
        else:
            item_keys.append((u["first"] - 1, key))
            consumed.add(key)
assert consumed == wanted_items, wanted_items - consumed

new_lines, texts, removed = splitlib.take(path, [k for _, k in item_keys] + method_keys)
out = lines[:10] + ["", f"//! {module_doc}", "", "use super::*;", ""]
before = [texts[k] for at, k in item_keys if at < main_impl]
after = [texts[k] for at, k in item_keys if at > main_impl]
parts = before
if method_keys:
    parts = parts + ["impl Interp {\n" + "\n\n".join(texts[k] for k in method_keys) + "\n}"]
parts = parts + after
out += ["\n\n".join(parts)]
child_path = os.path.join(src_dir, "run", f"{child}.rs")
assert not os.path.exists(child_path)
open(child_path, "w").write("\n".join(out) + "\n")
open(path, "w").write("\n".join(new_lines) + "\n")
splitlib.write_json(removed_json, removed)
print(f"{child}: {len(item_keys)} items, {len(method_keys)} members, {len(removed)} lines removed")
