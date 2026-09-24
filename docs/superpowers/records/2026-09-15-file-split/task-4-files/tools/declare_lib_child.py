"""Declares `mod CHILD;` in lib.rs, with a one-line comment above it, after
the last module declaration this task added at the head of the file
(`variables`).

usage: declare_lib_child.py SRC_DIR CHILD "COMMENT"
"""
import sys
src, child, comment = sys.argv[1:4]
path = f"{src}/lib.rs"
text = open(path).read()
anchor = "\nmod variables;\n"
assert text.count(anchor) == 1
text = text.replace(anchor, anchor + f"\n// {comment}\nmod {child};\n")
open(path, "w").write(text)
