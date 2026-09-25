"""A declared edit outside a commit's parent and destinations, checked
against its rule by comparing HEAD's text with the working tree's (Task 5's
`other_edits.py` `subst:` rule, for any path under rust/, `corpus/`
included, which that tool's `git status -- crates` does not see):

  PATH=REGEX=>REPL[;;REGEX=>REPL...]
      HEAD's text with each REGEX replaced by REPL, in order, equals the
      working text byte for byte; each substitution must match exactly once
      (so it cannot silently widen), and together they must change
      something.

Prints one line per path and "extra edits: K files, all as declared" or
"extra edits: FAILED". Exit 1 on any failure.

usage: extra_edits10.py RUST_DIR OUT PATH=RULE...
"""
import re, subprocess, sys
rust, out = sys.argv[1:3]
lines, bad = [], 0
for arg in sys.argv[3:]:
    path, rule = arg.split("=", 1)
    head = subprocess.run(["git", "-C", rust, "show", f"HEAD:rust/{path}"], capture_output=True, text=True, check=True).stdout
    work = open(f"{rust}/{path}").read()
    text, notes, ok = head, [], True
    for sub in rule.split(";;"):
        pat, repl = sub.split("=>", 1)
        text, n = re.subn(pat, repl.replace("\\n", "\n"), text)
        notes.append(f"{n} occurrence(s) of {pat!r} -> {repl!r}")
        ok &= n == 1
    same = text == work
    changed = head != work
    ok &= same and changed
    bad += not ok
    lines.append(f"{path}: {'OK' if ok else 'FAIL'} {'; '.join(notes)}; HEAD with them applied {'equals' if same else 'DIFFERS from'} the working text; {'changed' if changed else 'UNCHANGED'}")
lines.append(f"extra edits: {len(sys.argv) - 3} files, all as declared" if not bad else "extra edits: FAILED")
open(out, "w").write("\n".join(lines) + "\n")
print("\n".join(lines))
sys.exit(1 if bad else 0)
