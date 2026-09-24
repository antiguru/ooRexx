"""Declares `mod CHILD;` in lib.rs, with a one-line comment above it, after
the last of the module declarations at the head of the file (`libraries`).

usage: declare_lib_child.py SRC_DIR CHILD "COMMENT"
"""
import sys
src, child, comment = sys.argv[1:4]
path = f"{src}/lib.rs"
text = open(path).read()
anchor = "mod libraries;\nuse libraries::Libraries;\n"
assert text.count(anchor) == 1
text = text.replace(anchor, anchor + f"\n// {comment}\nmod {child};\n")
open(path, "w").write(text)
