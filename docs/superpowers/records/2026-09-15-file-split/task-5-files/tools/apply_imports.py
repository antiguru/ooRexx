"""Write the imports parent_side.py found, and widen the child items the rest
of `dispatch` reaches from private to `pub(super)` (an item already
`pub(crate)` keeps it).

usage: apply_imports.py SRC_DIR CHILD "CHILD NAMES" "PARENT NAMES" "CRATE NAMES" "METHODS"
"""
import os, re, sys
PARENT = os.environ.get("SPLIT_PARENT", "dispatch")
PARENT_RS = PARENT + ".rs"
src, child, own, parent, crate, methods = sys.argv[1:7]
own, parent, crate, methods = (x.split() for x in (own, parent, crate, methods))
cpath = f"{src}/{PARENT}/{child}.rs"
c = open(cpath).read()
if own:
    m = re.search(r"\n//! [^\n]*(\n//![^\n]*)*\n\n", c)
    c = c[: m.end()] + "use super::{" + ", ".join(own) + "};\n" + c[m.end():]
for name in parent + crate:
    pat = re.compile(r"^(fn|const|static|struct|enum|type) " + re.escape(name) + r"\b", re.M)
    hits = pat.findall(c)
    already = re.search(r"^pub\([a-z]+\) (fn|const|static|struct|enum|type) " + re.escape(name) + r"\b", c, re.M)
    assert len(hits) + (1 if already else 0) == 1, (name, hits, bool(already))
    c = pat.sub(lambda m: "pub(super) " + m.group(0), c)
for name in methods:
    pat = re.compile(r"^    fn " + re.escape(name) + r"\b", re.M)
    assert len(pat.findall(c)) == 1, name
    c = pat.sub("    pub(super) fn " + name, c)
open(cpath, "w").write(c)
d = open(f"{src}/{PARENT_RS}").read()
decl = f"\nmod {child};\n"
assert d.count(decl) == 1
lines = ""
if crate:
    lines += f"pub(crate) use {child}::" + (crate[0] if len(crate) == 1 else "{" + ", ".join(crate) + "}") + ";\n"
if parent:
    lines += f"use {child}::" + (parent[0] if len(parent) == 1 else "{" + ", ".join(parent) + "}") + ";\n"
d = d.replace(decl, decl + lines)
open(f"{src}/{PARENT_RS}", "w").write(d)
