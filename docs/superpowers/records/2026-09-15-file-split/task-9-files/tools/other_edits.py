"""Checks each edit a commit makes outside its parent and destinations
against the one rule declared for that file, comparing HEAD's text with the
working tree's:

  subst:REGEX=>REPL   HEAD's text with REGEX replaced by REPL, everywhere,
                      equals the working text byte for byte (cmp); several
                      substitutions, applied in order, are joined with `;;`,
                      and `\\n` in a REPL is a newline;
  substsort:REGEX=>REPL
                      the same, after sorting each run of consecutive
                      single-line `use` declarations on both sides (rustfmt
                      re-sorts an import the rename moved in the order);
  insert              the working text is HEAD's with lines inserted only.
  narrow              only names dropped from `use` lists; every other line
                      identical, in order (`narrow+insert`: whole lines may
                      also be inserted, and their names are listed);
  imports:OLD=>NEW    only top-level `use`/`mod` declarations (and the `//`
                      comment lines directly above an inserted `mod`) differ:
                      every other line is identical in order (cmp); the
                      imported names, as a multiset of leaf names, are the
                      same; and every name whose path changed moved from a
                      path starting OLD to one starting NEW.

Every file the working tree changes under rust/crates must have a rule
(the parent and destinations are given as `instruments`, meaning instruments
1-3 cover them), and every rule must change something.

usage: other_edits.py RUST_DIR OUT PATH=RULE...
"""
import difflib, os, re, subprocess, sys
rust, out = sys.argv[1:3]
rules = dict(a.split("=", 1) for a in sys.argv[3:])
changed = subprocess.run(["git", "-C", rust, "status", "--porcelain", "-uall", "--", "crates"], capture_output=True, text=True).stdout.split("\n")
changed = sorted(l[3:].removeprefix("rust/") for l in changed if l)
lines, bad = [], 0


def sort_use_runs(text):
    out, run = [], []
    for l in text.split("\n"):
        if l.startswith("use ") and l.endswith(";"):
            run.append(l)
            continue
        out += sorted(run) + [l]
        run = []
    return "\n".join(out + sorted(run))

def decl_spans(text):
    """0-based line indices of top-level `use`/`mod` declarations (a `use`
    may span lines up to its `;`), and the leaf names each `use` imports."""
    ls = text.split("\n")
    idx, names = set(), []
    i = 0
    while i < len(ls):
        l = ls[i]
        m = re.match(r"(pub(\([a-z:_ ]+\))? )?(use|mod) ", l)
        if m:
            if m.group(3) == "mod" and not l.rstrip().endswith(";"):
                # an inline module, which is code, not a declaration
                i += 1
                continue
            j = i
            while not ls[j].rstrip().endswith(";"):
                j += 1
            body = " ".join(x.strip() for x in ls[i : j + 1])
            if m.group(3) == "use":
                u = re.sub(r"^(pub(\([a-z:_ ]+\))? )?use ", "", body).rstrip(";").replace(" ", "")
                mm = re.fullmatch(r"([\w:]+)::\{(.*)\}", u)
                leaves = [(mm.group(1), n) for n in mm.group(2).split(",") if n] if mm else [(u.rsplit("::", 1)[0], u.rsplit("::", 1)[-1])]
                names += leaves
            idx.update(range(i, j + 1))
            i = j + 1
            continue
        i += 1
    return ls, idx, names


def split_comments(ls, idx):
    """Lines outside the declarations, with the `//` comment lines directly
    above a declaration set aside."""
    rest, comments = [], []
    for k, l in enumerate(ls):
        if k in idx:
            continue
        nxt = k
        while nxt < len(ls) and ls[nxt].lstrip().startswith("//") and nxt not in idx:
            nxt += 1
        if l.lstrip().startswith("//") and nxt < len(ls) and nxt in idx:
            comments.append(l)
        else:
            rest.append(l)
    return rest, comments


def imports_rule(head, now, old, new, allowed_comment_edits):
    from collections import Counter
    a, ai, an = decl_spans(head)
    b, bi, bn = decl_spans(now)
    notes = []
    ka, ca_c = split_comments(a, ai)
    kb, cb_c = split_comments(b, bi)
    ops = difflib.SequenceMatcher(a=ka, b=kb, autojunk=False).get_opcodes()
    blanks = sum(j2 - j1 for t, i1, i2, j1, j2 in ops if t == "insert" and not any(kb[j1:j2]))
    ok = all(t == "equal" or (t == "insert" and not any(kb[j1:j2])) for t, i1, i2, j1, j2 in ops)
    if not ok:
        for d in difflib.unified_diff(ka, kb, "head", "now", lineterm="", n=0):
            notes.append("REST " + d)
    else:
        notes.append(f"{len(ka)} lines outside the declarations identical, in order, with {blanks} blank lines inserted")
    removed_c = Counter(ca_c) - Counter(cb_c)
    added_c = Counter(cb_c) - Counter(ca_c)
    for c in removed_c.elements():
        notes.append(f"comment removed or edited: {c.strip()}")
    for c in added_c.elements():
        notes.append(f"comment added: {c.strip()}")
    if sum(removed_c.values()) > allowed_comment_edits:
        ok = False
        notes.append(f"FAIL {sum(removed_c.values())} comment lines removed or edited, {allowed_comment_edits} declared")
    na = sorted(n for _, n in an)
    nb = sorted(n for _, n in bn)
    if na != nb:
        ok = False
        notes.append(f"imported leaf names differ: only head {sorted(set(na)-set(nb))}, only now {sorted(set(nb)-set(na))}")
    pa, pb = {}, {}
    for p, n in an:
        pa.setdefault(n, []).append(p)
    for p, n in bn:
        pb.setdefault(n, []).append(p)
    for n in sorted(pa):
        if sorted(pa[n]) != sorted(pb.get(n, [])):
            moved_ok = len(pa[n]) == 1 and len(pb.get(n, [])) == 1 and pa[n][0].startswith(old) and pb[n][0].startswith(new)
            notes.append(f"{'moved' if moved_ok else 'BAD'} {n}: {pa[n]} -> {pb.get(n)}")
            ok = ok and moved_ok
    ismod = lambda l: re.match(r"(pub(\([a-z:_ ]+\))? )?mod ", l)
    mods_a = sorted(a[k] for k in ai if ismod(a[k]))
    mods_b = sorted(b[k] for k in bi if ismod(b[k]))
    for m in sorted(set(mods_b) - set(mods_a)):
        notes.append(f"added {m}")
    for m in sorted(set(mods_a) - set(mods_b)):
        notes.append(f"FAIL removed {m}"); ok = False
    return ok, notes

def narrow_rule(head, now, allow_insert):
    from collections import Counter
    a, ai, an = decl_spans(head)
    b, bi, bn = decl_spans(now)
    notes = []
    ka = [l for k, l in enumerate(a) if k not in ai]
    kb = [l for k, l in enumerate(b) if k not in bi]
    ops = difflib.SequenceMatcher(a=ka, b=kb, autojunk=False).get_opcodes()
    ok = all(t == "equal" or (allow_insert and t == "insert") for t, *_ in ops)
    for t, i1, i2, j1, j2 in ops:
        if t == "insert" and allow_insert:
            for l in kb[j1:j2]:
                notes.append(f"inserted: {l}")
        elif t != "equal":
            ok = False
            notes.append(f"REST {t} {ka[i1:i2]!r} -> {kb[j1:j2]!r}")
    ca, cb = Counter(an), Counter(bn)
    extra = cb - ca
    if extra and not allow_insert:
        ok = False
    for (p, n), c in sorted(extra.items()):
        notes.append(f"{'added (inserted declaration)' if allow_insert else 'BAD added'} {p}::{n}")
    for (p, n), c in sorted((ca - cb).items()):
        notes.append(f"dropped {p}::{n}")
    return ok, notes

full = lambda r: r if r.startswith("crates/") else f"crates/{os.environ.get('SPLIT_CRATE', 'rexx-exec')}/src/" + r
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
    if rule.startswith(("subst:", "substsort:")):
        # Several substitutions, applied in order, are joined with `;;`.
        a, b, ok, desc = head, now, True, []
        for part in rule.split(":", 1)[1].split(";;"):
            pat, repl = part.split("=>", 1)
            repl = repl.replace("\\n", "\n")
            n = len(re.findall(pat, a))
            ok = ok and n > 0
            desc.append(f"{n} occurrences of {pat!r} -> {repl!r}")
            a = re.sub(pat, repl, a)
        if rule.startswith("substsort:"):
            a, b = sort_use_runs(a), sort_use_runs(b)
        ok = ok and a == b
        lines.append(f"{path}: {'OK' if ok else 'FAIL'} {'; '.join(desc)}; HEAD with them replaced {'equals' if ok else 'DIFFERS from'} the working text")
    elif rule == "insert":
        a, b = head.split("\n"), now.split("\n")
        ops = difflib.SequenceMatcher(a=a, b=b, autojunk=False).get_opcodes()
        ins = [(j1, b[j1:j2]) for t, i1, i2, j1, j2 in ops if t == "insert"]
        ok = all(t in ("equal", "insert") for t, *_ in ops) and ins
        lines.append(f"{path}: {'OK' if ok else 'FAIL'} inserted lines only")
        for j, block in ins:
            for k, l in enumerate(block):
                lines.append(f"  +{j + k + 1}: {l}")
    elif rule == "narrow" or rule.startswith("narrow+"):
        # Import lists that lost names and nothing else; `narrow+insert`
        # also lets whole lines be inserted, as another rule's `insert`.
        ok, notes = narrow_rule(head, now, rule == "narrow+insert")
        lines.append(f"{path}: {'OK' if ok else 'FAIL'} import lists narrowed{' and lines inserted' if rule == 'narrow+insert' else ''}, nothing else")
        lines += ["  " + n for n in notes]
    elif rule.startswith("imports:"):
        spec = rule[len("imports:"):]
        allowed = 0
        if ":comments=" in spec:
            spec, allowed = spec.split(":comments=")
        old, new = spec.split("=>")
        ok, notes = imports_rule(head, now, old, new, int(allowed))
        lines.append(f"{path}: {'OK' if ok else 'FAIL'} import/mod lines only, names moved {old} -> {new}")
        lines += ["  " + n for n in notes]
    else:
        ok = False; lines.append(f"{path}: FAIL unknown rule {rule}")
    bad += not ok
lines.append("PASS" if not bad else f"FAILURES: {bad}")
open(out, "w").write("\n".join(lines) + "\n")
print("\n".join(lines))
sys.exit(1 if bad else 0)
