"""Widens the private `impl Interp` members of a new crate-root child that
the rest of the crate now fails to reach, from private to `pub(super)`
(which for a child of the crate root is the whole crate): compile, collect
every "method `x` is private" and "associated function `x` is private",
widen, repeat until none is left. A member already `pub(crate)` keeps it.

usage: widen_methods.py RUST_DIR TARGET CHILD_RS
"""
import os, re, subprocess, sys
rust, target, child = sys.argv[1:4]
path = f"{rust}/crates/rexx-exec/src/{child}"
widened = []
while True:
    r = subprocess.run(["cargo", "check", "-j", "8", "-p", "rexx-exec", "--tests", "--message-format=short"],
                       cwd=rust, env={**os.environ, "CARGO_TARGET_DIR": target}, capture_output=True, text=True)
    names = set(re.findall(r"(?:method|associated function) `(\w+)` is private", r.stderr))
    if not names:
        break
    c = open(path).read()
    for n in sorted(names):
        pat = re.compile(r"^    fn " + re.escape(n) + r"\b", re.M)
        assert len(pat.findall(c)) == 1, n
        c = pat.sub("    pub(super) fn " + n, c)
        widened.append(n)
    open(path, "w").write(c)
print("WIDENED", sorted(widened))
for line in r.stderr.splitlines():
    if "error" in line or "warning:" in line:
        print("REST", line)
