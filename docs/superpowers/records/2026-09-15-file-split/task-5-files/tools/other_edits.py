"""Checks each edit a commit makes outside its parent and destinations
against the one rule declared for that file, comparing HEAD's text with the
working tree's:

  subst:REGEX=>REPL   HEAD's text with REGEX replaced by REPL, everywhere,
                      equals the working text byte for byte (cmp);
  insert              the working text is HEAD's with lines inserted only.

Every file the working tree changes under rust/crates must have a rule
(the parent and destinations are given as `instruments`, meaning instruments
1-3 cover them), and every rule must change something.

usage: other_edits.py RUST_DIR OUT PATH=RULE...
"""
import difflib, re, subprocess, sys
rust, out = sys.argv[1:3]
rules = dict(a.split("=", 1) for a in sys.argv[3:])
changed = subprocess.run(["git", "-C", rust, "status", "--porcelain", "-uall", "--", "crates"], capture_output=True, text=True).stdout.split("\n")
changed = sorted(l[3:].removeprefix("rust/") for l in changed if l)
lines, bad = [], 0
full = lambda r: r if r.startswith("crates/") else "crates/rexx-exec/src/" + r
unruled = [c for c in changed if c not in {full(r) for r in rules}]
for c in unruled:
    lines.append(f"FAIL no rule for changed path {c}"); bad += 1
for rel, rule in rules.items():
    if rule == "instruments":
        lines.append(f"{rel}: covered by instruments 1-3")
        continue
    path = full(rel)
    head = subprocess.run(["git", "-C", rust, "show", f"HEAD:rust/{path}"], capture_output=True, text=True, check=True).stdout
    now = open(f"{rust}/{path}").read()
    if rule.startswith("subst:"):
        pat, repl = rule[len("subst:"):].split("=>", 1)
        n = len(re.findall(pat, head))
        ok = n > 0 and re.sub(pat, repl, head) == now
        lines.append(f"{path}: {'OK' if ok else 'FAIL'} {n} occurrences of {pat!r} -> {repl!r}; HEAD with them replaced {'equals' if ok else 'DIFFERS from'} the working text")
    elif rule == "insert":
        a, b = head.split("\n"), now.split("\n")
        ops = difflib.SequenceMatcher(a=a, b=b, autojunk=False).get_opcodes()
        ins = [(j1, b[j1:j2]) for t, i1, i2, j1, j2 in ops if t == "insert"]
        ok = all(t in ("equal", "insert") for t, *_ in ops) and ins
        lines.append(f"{path}: {'OK' if ok else 'FAIL'} inserted lines only")
        for j, block in ins:
            for k, l in enumerate(block):
                lines.append(f"  +{j + k + 1}: {l}")
    else:
        ok = False; lines.append(f"{path}: FAIL unknown rule {rule}")
    bad += not ok
lines.append("PASS" if not bad else f"FAILURES: {bad}")
open(out, "w").write("\n".join(lines) + "\n")
print("\n".join(lines))
sys.exit(1 if bad else 0)
