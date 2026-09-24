"""Replace a new child's `use super::*;` with the explicit `use super::{..}`
list the compiler asks for: remove the glob, compile, and collect every name
reported unresolved in that file.

usage: explicit_imports.py RUST_DIR TARGET CHILD
"""
import re, subprocess, sys
rust, target, child = sys.argv[1:4]
path = f"{rust}/crates/rexx-exec/src/dispatch/{child}.rs"
text = open(path).read()
assert text.count("\nuse super::*;\n") == 1
open(path, "w").write(text.replace("\nuse super::*;\n", "\n"))
r = subprocess.run(["cargo", "check", "-j", "8", "-p", "rexx-exec", "--tests", "--message-format=short"],
                   cwd=rust, env={**__import__("os").environ, "CARGO_TARGET_DIR": target},
                   capture_output=True, text=True)
names = set()
other = []
for line in r.stderr.splitlines():
    if f"dispatch/{child}.rs" not in line or "error" not in line:
        if "error[" in line:
            other.append(line)
        continue
    m = re.search(r"(?:cannot find (?:value|function|type|struct|trait|macro|struct, variant or union type|tuple struct or tuple variant) `([^`]+)`|failed to resolve: use of undeclared (?:type|crate or module) `([^`]+)`|use of undeclared type `([^`]+)`)", line)
    if m:
        names.add(next(g for g in m.groups() if g))
    else:
        other.append(line)
names = sorted(names, key=lambda n: (n[0].islower(), n))
print("NAMES", names)
for o in other:
    print("OTHER", o)
