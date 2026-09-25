"""Negative controls for this task's tooling change (part A, on Task 8's c2,
runnable before this task moves anything) and for its own commits (part B,
run once they exist). Trees come from `git archive`; each plant is made in
its own copy.

Part A, the widened positional-word list (Task 8's review, M2: a planted
"The converters follow the table." in a parent went unlisted). For each word
the widening adds, a one-line comment using it is planted in Task 8's c2
parent (`values.rs`), and one in its destination (`values/convert.rs`):

  A-<word>. this task's comments7.py lists the planted line under (c), and
            Task 8's committed comments7.py, on the same trees, does not
            (the gap), while it did run its (c) section;
  A-none.   a planted line with no listed word is listed by neither, so the
            listing keys on the word and not on the plant;
  A-positional. this task's positional.py reports a planted comment in the
            parent that names a moved converter with "follow", and Task 8's
            does not.

Part B, instruments 1-3 on this task's own commits (a control passes when
instruments.py exits 1 and reports at least the instruments listed); see
PART_B below.

usage: controls_task9.py
"""
import os, re, shutil, subprocess, sys
os.environ["PYTHONDONTWRITEBYTECODE"] = "1"
S = "/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-9"
M = "/home/moritz/dev/repos/ooRexx-rust-rewrite"
REC = f"{M}/docs/superpowers/records/2026-09-15-file-split"
here = os.path.dirname(os.path.abspath(__file__))
T8 = f"{REC}/task-8-files/tools"
work = f"{S}/ctl-work/task9"
ITEM_TOOL = os.path.join(here, "item-tool", "target", "release", "item-tool")
os.makedirs(work, exist_ok=True)


def tree(c, what="rust/crates/rexx-exec/src"):
    d = f"{S}/trees/{c}"
    if not os.path.isdir(f"{d}/rust/crates"):
        shutil.rmtree(d, ignore_errors=True)
        os.makedirs(d)
        a = subprocess.run(["git", "-C", M, "archive", c, "rust"], capture_output=True, check=True).stdout
        subprocess.run(["tar", "-x", "-C", d], input=a, check=True)
    return f"{d}/{what}"


def parent(c):
    return subprocess.run(["git", "-C", M, "rev-parse", "--short=9", c + "^"], capture_output=True, text=True, check=True).stdout.strip()


def commit(task, n, base):
    out = subprocess.run(["git", "-C", M, "log", "--format=%h", f"--grep=task-{task}-files/c{n}/", "-F", f"{base}..HEAD"], capture_output=True, text=True, check=True).stdout.split()
    assert len(out) <= 1, (task, n, out)
    return out[0] if out else None


missed = total = 0


def report(name, ok, detail):
    global missed, total
    total += 1
    missed += not ok
    print(f"control {name}: {'CAUGHT' if ok else 'MISSED'}")
    print("  " + detail.strip().replace("\n", "\n  "))


def plant(src, dst, rel, old, new):
    shutil.rmtree(dst, ignore_errors=True)
    shutil.copytree(src, dst)
    p = os.path.join(dst, rel)
    s = open(p).read()
    assert s.count(old) == 1, (rel, old, s.count(old))
    open(p, "w").write(s.replace(old, new))
    return dst


def instruments(tool_dir, pre, post, removed, env, extra):
    r = subprocess.run([sys.executable, os.path.join(tool_dir, "instruments.py"), pre, post, removed, post + "-out"] + extra, capture_output=True, text=True, env=dict(env, ITEM_TOOL=ITEM_TOOL))
    tags = sorted({l.split(" ")[0] for l in r.stdout.splitlines() if l[:2] in ("I1", "I2", "I3")})
    return r.returncode, tags, (r.stdout + r.stderr).strip()


# ------------------------------------------------------------------ part A
C2 = "526f14761"
assert commit(8, 2, "5c7173fac") == C2
PARENT_ANCHOR = ("crates/rexx-api/src/values.rs", "\npub fn pointer_string(")
DEST_ANCHOR = ("crates/rexx-api/src/values/convert.rs", "\npub(super) fn int8_from_native(")
WORDS = [
    ("before", "the converters come before the table"),
    ("after", "the converters come after the table"),
    ("ahead", "the table sits ahead of the converters"),
    ("behind", "the converters sit behind the table"),
    ("follow", "the converters follow the table"),
    ("follows", "the table follows the codes"),
    ("followed", "the codes are followed by the table"),
    ("precede", "the codes precede the table"),
    ("precedes", "the table precedes the converters"),
    ("end of", "the converters sit at the end of the list"),
    ("top of", "the codes sit at the top of the list"),
    ("bottom of", "the converters sit at the bottom of the list"),
    ("start of", "the codes sit at the start of the list"),
]
cenv = dict(os.environ, ITEM_TOOL=ITEM_TOOL, SPLIT_CRATE="rexx-api")
pre_src = tree(parent(C2), "rust/crates/rexx-api/src")


def listed(tool_dir, post_rust, marker):
    args = [pre_src, post_rust, "values.rs", "values/convert.rs"]
    o = subprocess.run([sys.executable, os.path.join(tool_dir, "comments7.py")] + args, capture_output=True, text=True, env=cenv)
    lines = o.stdout.splitlines()
    ran = [l for l in lines if l.startswith("(c) ")]
    # only the (c) section's lines
    c_at = next((i for i, l in enumerate(lines) if l.startswith("(c) ")), len(lines))
    hits = [l for l in lines[c_at:] if marker in l]
    return hits, ran, o.returncode


for where, (rel, anchor) in (("parent", PARENT_ANCHOR), ("destination", DEST_ANCHOR)):
    for word, sentence in WORDS + [("none", "the converters and the table")]:
        marker = f"Planted-{where}-{word.replace(' ', '_')}"
        d = plant(tree(C2, "rust"), f"{work}/a-{where}-{word.replace(' ', '_')}", rel, anchor, f"\n// {marker}: {sentence}.{anchor}")
        new, ran_new, rc_new = listed(here, d, marker)
        old, ran_old, rc_old = listed(T8, d, marker)
        if word == "none":
            ok = new == [] and old == [] and ran_new != [] and ran_old != [] and rc_new == rc_old == 0
        else:
            ok = len(new) == 1 and rel.split("/src/")[1] in new[0] and old == [] and ran_old != [] and rc_new == rc_old == 0
        report(f"A-{word} ({where})", ok, f"this task's comments7.py (c): {new} under {ran_new}\nTask 8's (c): {old} under {ran_old}")

# positional.py, which lists a comment block naming a moved unit with a
# positional word: a planted "follow" naming the moved `int8_from_native`.
d = plant(tree(C2, "rust"), f"{work}/a-positional", PARENT_ANCHOR[0], PARENT_ANCHOR[1],
          f"\n// Planted: `int8_from_native` and the other converters follow the table.{PARENT_ANCHOR[1]}")
penv = dict(cenv, SPLIT_PARENT_RS="values.rs", SPLIT_DESTS="values/convert.rs")
pos = lambda t: subprocess.run([sys.executable, os.path.join(t, "positional.py"), pre_src, d + "/crates/rexx-api/src"], capture_output=True, text=True, env=penv).stdout
pn, po = pos(here), pos(T8)
hit = lambda o: [l for l in o.splitlines() if "int8_from_native" in l]
report("A-positional: positional.py lists a planted `follow` naming a moved unit", len(hit(pn)) == 1 and hit(po) == [] and po.strip().endswith("done"),
       f"this task's positional.py: {hit(pn)}\nTask 8's: {hit(po)} (ran: {po.strip().splitlines()[-1:]})")

# ------------------------------------------------------------------ part B
BASE = "72f2bc3d8"


def first(old, new):
    return lambda s: s.replace(old, new, 1) if old in s else s


def swap(x, y):
    def f(s):
        assert s.count(x) == 1 and s.count(y) == 1, (x, y)
        return s.replace(x, "\0").replace(y, x).replace("\0", y)
    return f


def delete_fn(doc_start):
    def f(s):
        a = s.index(doc_start)
        b = s.index("\n}\n", a) + 3
        return s[:a] + s[b + 1:]
    return f


# (n, parent, destinations, instrument options, plants); a plant is
# (name, instruments expected, file, text -> planted text).
PART_B = [
    (1, "block.rs", "block/references.rs", [], [
        ("B1-c1 a moved fn's body changes a token", ["I1b", "I2"], "block/references.rs",
         first("visit_expr(target, symbols, f);", "visit_expr(value, symbols, f);")),
        ("B2-c1 a moved helper is deleted", ["I2"], "block/references.rs",
         delete_fn("/// A bare variable slot")),
        ("B3-c1 an unmoved line of block.rs changes", ["I1a"], "block.rs",
         first("self.referenced.insert(Box::from(name.as_bytes()));", "self.referenced.insert(Box::from(name.as_bytes() ));")),
        ("B4-c1 two moved helpers swap their doc comments", ["I1b", "I2", "I3"], "block/references.rs",
         swap("/// A bare variable slot, which is a name and not an expression.", "/// Calls `f` with the name of every variable reference in one expression.")),
    ]),
]
for n, parent_rs, dests, opts, plants in PART_B:
    c = commit(9, n, BASE)
    if c is None:
        print(f"part B c{n}: no commit yet, skipped")
        continue
    pre_t, post_t = tree(parent(c), "rust/crates/rexx-parse/src"), tree(c, "rust/crates/rexx-parse/src")
    removed = f"{REC}/task-9-files/c{n}/removed.json"
    env = dict(os.environ, SPLIT_CRATE="rexx-parse", SPLIT_PARENT=parent_rs[:-3], SPLIT_PARENT_RS=parent_rs, SPLIT_DESTS=dests, SPLIT_TESTS_RS="none")
    rc, tags, out = instruments(here, pre_t, post_t, removed, env, opts)
    report(f"B c{n} as committed passes", rc == 0, f"exit {rc}, reported {tags}")
    for name, expect, rel, fn in plants:
        d = f"{work}/b-c{n}-{name.split()[0]}"
        shutil.rmtree(d, ignore_errors=True)
        shutil.copytree(post_t, d)
        p = os.path.join(d, rel)
        s = open(p).read()
        s2 = fn(s)
        assert s2 != s, name
        open(p, "w").write(s2)
        rc, tags, out = instruments(here, pre_t, d, removed, env, opts)
        ok = rc == 1 and set(expect) <= set(tags)
        report(f"{name} (c{n} {c})", ok, f"exit {rc}, expected {expect}, reported {tags}\n" + "\n".join(l for l in out.splitlines() if l[:2] in ("I1", "I2", "I3"))[:3000])

print(f"{total - missed} of {total} controls caught as expected")
sys.exit(1 if missed else 0)
