"""Puts one import block directly after a new child's module doc, replacing
whatever `use` lines stand there (the mover's `use super::*;` placeholder,
or nothing once explicit_imports.py removed it).

usage: set_imports.py CHILD_PATH "use ...;" ["use ...;" ...]
"""
import re, sys
path, uses = sys.argv[1], sys.argv[2:]
s = open(path).read()
m = re.search(r"\n//! [^\n]*(\n//![^\n]*)*\n", s)
rest = re.sub(r"^\n*(use [^\n]*\n)*\n*", "", s[m.end():], count=1)
open(path, "w").write(s[: m.end()] + "\n" + "".join(u + "\n" for u in uses) + "\n" + rest)
