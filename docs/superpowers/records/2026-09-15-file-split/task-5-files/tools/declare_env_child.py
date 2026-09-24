"""Declares `mod CHILD;` in environment.rs, with a one-line comment above it,
after the file's imports (or after the last child this task declared there),
and sets the child's module doc in place of the mover's placeholder.

usage: declare_env_child.py SRC_DIR CHILD "COMMENT" "MODULE DOC"
"""
import sys
src, child, comment, doc = sys.argv[1:5]
path = f"{src}/environment.rs"
text = open(path).read()
anchor = "\nuse crate::{Failure, Interp, Loud};\n"
for prior in ("route",):
    if f"\nmod {prior};\n" in text:
        anchor = f"\nmod {prior};\n"
assert text.count(anchor) == 1
text = text.replace(anchor, anchor + f"\n// {comment}\nmod {child};\n")
open(path, "w").write(text)
cpath = f"{src}/environment/{child}.rs"
c = open(cpath).read()
assert c.count("\n//! x\n") == 1
open(cpath, "w").write(c.replace("\n//! x\n", "\n" + "\n".join("//! " + l for l in doc.split("\n")) + "\n", 1))
