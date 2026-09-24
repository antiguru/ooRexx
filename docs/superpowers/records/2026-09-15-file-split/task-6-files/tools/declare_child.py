"""Declares `mod CHILD;` in run.rs, with a one-line comment above it, in the
block of child declarations that sits between the imports and `Flow`.

usage: declare_child.py SRC_DIR CHILD "COMMENT"
"""
import sys
src, child, comment = sys.argv[1:4]
path = f"{src}/run.rs"
text = open(path).read()
anchor = "/// Where control goes after one instruction"
assert text.count(anchor) == 1
text = text.replace(anchor, f"// {comment}\nmod {child};\n\n" + anchor)
open(path, "w").write(text)
