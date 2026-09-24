"""Compile with the new child declared, and list what the rest of the crate
now fails to reach in it: names to import into dispatch.rs (and widen to
`pub(super)` in the child), names reached by a crate path (`pub(crate) use`),
and private methods (widen).

usage: parent_side.py RUST_DIR TARGET CHILD

Errors inside the child itself are the child's own imports (CHILD line).
"""
import os, re, subprocess, sys
rust, target, child = sys.argv[1:4]
r = subprocess.run(["cargo", "check", "-j", "8", "-p", "rexx-exec", "--tests", "--message-format=short"],
                   cwd=rust, env={**os.environ, "CARGO_TARGET_DIR": target}, capture_output=True, text=True)
parent, crate_path, methods, other, own = set(), set(), set(), [], set()
for line in r.stderr.splitlines():
    if "error" not in line:
        continue
    path = line.split(":")[0]
    in_dispatch = "src/dispatch" in path
    if path.endswith(f"dispatch/{child}.rs"):
        m = re.search(r"cannot find \w+(?: \w+)* `(\w+)` in this scope", line)
        if m:
            own.add(m.group(1))
            continue
    found = False
    for m in re.finditer(r"`super::(\w+)`", line):
        parent.add(m.group(1)); found = True
    m = re.search(r"cannot find \w+(?: \w+)* `(\w+)` in (this scope|module `super`)", line)
    if m:
        parent.add(m.group(1)); found = True
    m = re.search(r"cannot find \w+(?: \w+)* `(\w+)` in module `(crate::)?dispatch`", line)
    if m:
        crate_path.add(m.group(1)); found = True
    for m in re.finditer(r"no `(\w+)` in `dispatch`", line):
        (parent if in_dispatch else crate_path).add(m.group(1)); found = True
    m = re.search(r"method `(\w+)` is private", line)
    if m:
        methods.add((m.group(1), "dispatch" if in_dispatch else "crate")); found = True
    m = re.search(r"no method named `(\w+)` found", line)
    if m:
        methods.add((m.group(1), "dispatch" if in_dispatch else "crate")); found = True
    if not found and "error[" in line:
        other.append(line)
print("CHILD", sorted(own, key=lambda n: (n[0].islower(), n)))
print("PARENT", sorted(parent))
print("CRATE", sorted(crate_path))
print("METHODS", sorted(methods))
for o in other:
    print("OTHER", o)
