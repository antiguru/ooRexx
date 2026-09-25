"""Drops the names rustc reports as unused imports in lib.rs from lib.rs's
own `use` declarations, and nothing else; a declaration left empty goes.
cargo fmt re-wraps what is left.

usage: narrow_imports.py RUST_DIR TARGET
"""
import os, re, subprocess, sys
rust, target = sys.argv[1:3]
path = f"{rust}/crates/rexx-exec/src/lib.rs"
r = subprocess.run(["cargo", "check", "-j", "8", "-p", "rexx-exec", "--tests", "--message-format=short"],
                   cwd=rust, env={**os.environ, "CARGO_TARGET_DIR": target}, capture_output=True, text=True)
unused = set()
for line in r.stderr.splitlines():
    if "src/lib.rs" in line and "unused import" in line:
        unused.update(re.findall(r"`(\w+)`", line.split("unused import", 1)[1]))
text = open(path).read()
def fix(m):
    base, names = m.group(1), m.group(2)
    kept = [n.strip() for n in names.split(",") if n.strip() and n.strip() not in unused]
    if not kept:
        return ""
    return f"use {base}::" + ("{" + ", ".join(kept) + "}" if len(kept) > 1 else kept[0]) + ";"
new = re.sub(r"(?ms)^use ([\w:]+)::\{([^}]*)\};", fix, text)
new = re.sub(r"(?m)^use ([\w:]+)::(\w+);", lambda m: "" if m.group(2) in unused else m.group(0), new)
open(path, "w").write(new)
print("DROPPED", sorted(unused))
