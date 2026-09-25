"""Replace a new crate-root child's `use super::*;` by explicit imports, in
the style of the crate root's other children: `use crate::{...}` for items
of the crate root, `use crate::<module>::X` for a name lib.rs itself imports
from one of its modules, and the original path for a name lib.rs imports
from another crate. The names are the ones the compiler reports unresolved
in the child once the glob is gone.

usage: imports.py RUST_DIR TARGET CHILD_RS
"""
import os, re, subprocess, sys
from collections import defaultdict
rust, target, child = sys.argv[1:4]
src = f"{rust}/crates/rexx-exec/src"
path = f"{src}/{child}"
text = open(path).read()
assert text.count("\nuse super::*;\n") == 1
open(path, "w").write(text.replace("\nuse super::*;\n", "\n"))
# where lib.rs imports each name from
lib = open(f"{src}/lib.rs").read()
origin = {}
for m in re.finditer(r"(?ms)^(?:pub(?:\([a-z]+\))? )?use ([a-z_0-9]+(?:::[a-z_0-9]+)*)::(\{[^;]*\}|\w+);", lib):
    base, names = m.group(1), m.group(2)
    for n in re.findall(r"\w+", names):
        origin[n] = base
mods = set(re.findall(r"(?m)^(?:pub(?:\([a-z]+\))? )?mod (\w+);", lib))
names = set()
for _ in range(6):
    r = subprocess.run(["cargo", "check", "-j", "8", "-p", "rexx-exec", "--tests", "--message-format=short"],
                       cwd=rust, env={**os.environ, "CARGO_TARGET_DIR": target}, capture_output=True, text=True)
    new = set()
    other = []
    for line in r.stderr.splitlines():
        if f"src/{child}" not in line or "error" not in line:
            continue
        m = re.search(r"(?:cannot find (?:value|function|type|struct|trait|macro|struct, variant or union type|tuple struct or tuple variant|derive macro|attribute macro) `([^`]+)`|failed to resolve: use of undeclared (?:type|crate or module) `([^`]+)`|use of undeclared type `([^`]+)`|cannot find [a-z ]+ `([^`]+)` in this scope)", line)
        if m:
            new.add(next(g for g in m.groups() if g))
        else:
            other.append(line)
    if not new:
        break
    names |= new
    groups = defaultdict(set)
    for n in names:
        base = origin.get(n)
        if base is None:
            groups["crate"].add(n)
        elif base.split("::")[0] in mods:
            groups["crate::" + base].add(n)
        else:
            groups[base].add(n)
    block = "".join(
        f"use {g}::" + (next(iter(ns)) if len(ns) == 1 else "{" + ", ".join(sorted(ns)) + "}") + ";\n"
        for g, ns in sorted(groups.items())
    )
    t = open(path).read()
    t = re.sub(r"(//! [^\n]*\n\n)(?:use [^\n]*\n)*", lambda m: m.group(1) + block, t, count=1)
    open(path, "w").write(t)
print("NAMES", sorted(names))
for o in other:
    print("OTHER", o)
