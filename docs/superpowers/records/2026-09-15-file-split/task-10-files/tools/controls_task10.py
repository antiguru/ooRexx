"""Negative controls for this task's tooling changes (part T) and for its own
commits (part B). Trees come from `git archive`; each plant is made in its
own copy under the work directory, never in the repository.

Part T, the tooling this task changed or added:

  T1   splitlib.strip_field_vis, called directly: a `pub(crate)` after a
       field's doc attribute is dropped (the Task 10 change), and one after
       a `]` that closes no attribute, or after an attribute standing
       after `;` rather than `{` or `,`, is kept.
  T2   On c1 as committed (a doc-commented field, `ClassRow::owner`,
       widened): Task 9's committed instruments.py and splitlib.py fail
       instrument 2 on `struct ClassRow` and on nothing else (the gap), and
       this task's pass.
  T3   On c1, that field's doc line changed as well as widened: caught by
       instruments 1 (b) and 3.
  T4   comments10.py's SPLIT_ROOT: a planted comment in `gate_table_c.rs`
       naming the moved `read_sections` is listed under (a) and (c) by
       comments10.py with SPLIT_ROOT=tests, and not at all by Task 9's
       comments7.py, which reads `src/` only and stops on the missing
       destination (the gap).
  T5   move_items10.py's widening, on a scratch copy of BASE's
       `gate_table_c.rs`: `^^struct Edge` puts `pub(crate) ` on the
       declaration and on every field line and changes nothing else;
       `+fn read_edges` puts `pub(super) `; an unprefixed key is verbatim.

Part B, instruments 1-3 on this task's own commits (a control passes when
instruments.py exits 1 and reports at least the instruments listed); see
PART_B below.

usage: controls_task10.py            (part T, and part B on every commit that exists)
       controls_task10.py --pre N    (part T, and part B for commit N over pre<N>/ and
                                      the working tree, before N is committed)
"""
import os, re, shutil, subprocess, sys
os.environ["PYTHONDONTWRITEBYTECODE"] = "1"
S = "/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-10"
M = "/home/moritz/dev/repos/ooRexx-rust-rewrite"
REC = f"{M}/docs/superpowers/records/2026-09-15-file-split"
here = os.path.dirname(os.path.abspath(__file__))
T9 = f"{REC}/task-9-files/tools"
work = f"{S}/ctl-work/task10"
ITEM_TOOL = os.path.join(here, "item-tool", "target", "release", "item-tool")
os.makedirs(work, exist_ok=True)
BASE = "8a1c43696"
sys.path.insert(0, here)
import splitlib


def tree(c, what="rust"):
    d = f"{S}/trees/{c}"
    if not os.path.isdir(f"{d}/rust/crates"):
        shutil.rmtree(d, ignore_errors=True)
        os.makedirs(d)
        a = subprocess.run(["git", "-C", M, "archive", c, "rust"], capture_output=True, check=True).stdout
        subprocess.run(["tar", "-x", "-C", d], input=a, check=True)
    return f"{d}/{what}"


def parent(c):
    return subprocess.run(["git", "-C", M, "rev-parse", "--short=9", c + "^"], capture_output=True, text=True, check=True).stdout.strip()


def commit(n):
    out = subprocess.run(["git", "-C", M, "log", "--format=%h", f"--grep=task-10-files/c{n}/", "-F", f"{BASE}..HEAD"], capture_output=True, text=True, check=True).stdout.split()
    assert len(out) <= 1, (n, out)
    return out[0] if out else None


missed = total = 0


def report(name, ok, detail):
    global missed, total
    total += 1
    missed += not ok
    print(f"control {name}: {'CAUGHT' if ok else 'MISSED'}")
    print("  " + detail.strip().replace("\n", "\n  "))


def plant_copy(src, dst, rel, fn):
    shutil.rmtree(dst, ignore_errors=True)
    # A `rust/` tree taken from the main checkout carries its target
    # directory, which no instrument reads.
    shutil.copytree(src, dst, ignore=shutil.ignore_patterns("target"))
    p = os.path.join(dst, rel)
    s = open(p).read()
    s2 = fn(s)
    assert s2 != s, (dst, rel)
    open(p, "w").write(s2)
    return dst


def first(old, new):
    def f(s):
        assert old in s, old
        return s.replace(old, new, 1)
    return f


def swap(x, y):
    def f(s):
        assert s.count(x) == 1 and s.count(y) == 1, (x, y)
        return s.replace(x, "\0").replace(y, x).replace("\0", y)
    return f


def delete_item(start, end="\n}\n"):
    def f(s):
        a = s.index(start)
        b = s.index(end, a) + len(end)
        return s[:a] + s[b + 1:]
    return f


def instruments(tool_dir, pre, post, removed, env, extra):
    # The output prefix is in the work directory, never beside POST: in
    # --pre mode POST is the main checkout's tree (Task 9 review, I1).
    out = os.path.join(work, "out-" + post.strip("/").replace("/", "_")[-80:])
    assert not os.path.realpath(out).startswith(os.path.realpath(M) + "/"), out
    r = subprocess.run([sys.executable, os.path.join(tool_dir, "instruments.py"), pre, post, removed, out] + extra, capture_output=True, text=True, env=dict(env, ITEM_TOOL=ITEM_TOOL))
    tags = sorted({l.split(" ")[0] for l in r.stdout.splitlines() if l[:2] in ("I1", "I2", "I3")})
    return r.returncode, tags, (r.stdout + r.stderr).strip()


pre_mode = sys.argv[1:2] == ["--pre"]


def pair(n):
    """(label, pre, post, removed, args, where) for commit n, or None."""
    if pre_mode and n == int(sys.argv[2]):
        crate, root = open(f"{S}/c{n}/where").read().split()
        return ("working tree", f"{S}/pre{n}", f"{M}/rust/crates/{crate}/{root}", f"{S}/c{n}/removed.json",
                open(f"{S}/c{n}/args").read().split("\n")[:-1], (crate, root))
    c = commit(n)
    if c is None:
        return None
    d = f"{REC}/task-10-files/c{n}"
    crate, root = open(f"{d}/where").read().split()
    return (c, tree(parent(c), f"rust/crates/{crate}/{root}"), tree(c, f"rust/crates/{crate}/{root}"), f"{d}/removed.json",
            open(f"{d}/args").read().split("\n")[:-1], (crate, root))


def env_for(args, where):
    parent_rs, dests = args[0], args[1]
    return dict(os.environ, SPLIT_CRATE=where[0], SPLIT_ROOT=where[1], SPLIT_PARENT=parent_rs[:-3],
                SPLIT_PARENT_RS=parent_rs, SPLIT_DESTS=dests, SPLIT_TESTS_RS="none")


# ------------------------------------------------------------------ part T
t = lambda x: x.split()
cases = [
    ("doc-commented field", 'struct A { a : u8 , # [ doc = "x" ] pub ( crate ) b : u8 , }', 'struct A { a : u8 , # [ doc = "x" ] b : u8 , }'),
    ("two attributes before a field", 'struct A { # [ doc = "x" ] # [ allow ] pub ( super ) b : u8 , }', 'struct A { # [ doc = "x" ] # [ allow ] b : u8 , }'),
    ("a `]` closing an index", "fn f ( ) { v [ 0 ] pub ( crate ) }", None),
    ("an attribute after `;`", "fn f ( ) { g ( x ) ; # [ allow ] pub ( crate ) fn h ( ) }", None),
]
bad = []
for name, before, after in cases:
    got = splitlib.strip_field_vis(t(before))
    want = t(after) if after else t(before)
    if got != want:
        bad.append(f"{name}: {' '.join(got)}")
report("T1 strip_field_vis drops a field's visibility after its attributes and nowhere else", not bad,
       "\n".join(bad) or "every case as expected: " + "; ".join(n for n, _, _ in cases))

p1 = pair(1)
if p1 is None:
    print("part T2-T4: no c1 yet, skipped")
else:
    label, pre_t, post_t, removed, args, where = p1
    env = env_for(args, where)
    rc9, tags9, out9 = instruments(T9, pre_t, post_t, removed, env, args[2:])
    rc10, tags10, out10 = instruments(here, pre_t, post_t, removed, env, args[2:])
    fails9 = [l for l in out9.splitlines() if l[:2] in ("I1", "I2", "I3")]
    report(f"T2 Task 9's instruments miss the doc-commented field this task's admit (c1, {label})",
           rc9 == 1 and fails9 == ["I2 token stream differs: struct ClassRow"] and rc10 == 0,
           f"Task 9's: exit {rc9}, {fails9}\nthis task's: exit {rc10}, {tags10}")
    d = plant_copy(post_t, f"{work}/t3", "table_c/rows.rs",
                   first("    /// The row set's `method-owner` column: the phase that owes this class's",
                         "    /// The row set's `method-owner` column: the phase that owes the class's"))
    rc, tags, out = instruments(here, pre_t, d, removed, env, args[2:])
    report(f"T3 a widened doc-commented field whose doc also changed (c1, {label})", rc == 1 and {"I1b", "I3"} <= set(tags),
           f"exit {rc}, expected I1b and I3, reported {tags}")
    # T4: the parent is under tests/, which comments7.py cannot see.
    rust = tree(label, "rust") if label != "working tree" else f"{M}/rust"
    d = plant_copy(rust, f"{work}/t4", "crates/rexx-exec/tests/gate_table_c.rs",
                   first("\nfn corpus_dir()", "\n// Planted: `read_sections` sits above the tests.\nfn corpus_dir()"))
    lst = lambda tool, e: subprocess.run([sys.executable, tool, pre_t, d, args[0], args[1]], capture_output=True, text=True, env=dict(e, ITEM_TOOL=ITEM_TOOL))
    new = lst(os.path.join(here, "comments10.py"), env)
    old = lst(os.path.join(T9, "comments7.py"), env)
    hit = lambda o: [l for l in o.stdout.splitlines() if "Planted" in l]
    # Task 9's reads the destinations under `src/`, where they do not
    # exist, so it cannot run on a `tests/` parent at all: the gap is that
    # it lists nothing, whichever way it stops.
    report("T4 comments10.py lists a planted comment in a tests/ parent; Task 9's comments7.py does not",
           len(hit(new)) == 2 and new.returncode == 0 and hit(old) == [],
           f"comments10.py: {hit(new)} (exit {new.returncode})\ncomments7.py: {hit(old)} (exit {old.returncode}; "
           f"{(old.stderr.strip().splitlines() or [''])[-1]})")

# T5: the mover's widening, on a scratch copy of BASE's parent.
src = tree(BASE, "rust/crates/rexx-exec/tests")
d = f"{work}/t5"
shutil.rmtree(d, ignore_errors=True)
shutil.copytree(src, d)
hdr = f"{work}/t5-header"
open(hdr, "w").write("//! planted header\n")
r = subprocess.run([sys.executable, os.path.join(here, "move_items10.py"), f"{d}/gate_table_c.rs", f"{d}/t5child/c.rs", f"{work}/t5-removed.json", hdr,
                    "^^struct Edge", "+fn read_edges", "fn read_table"], capture_output=True, text=True, env=dict(os.environ, ITEM_TOOL=ITEM_TOOL))
child = open(f"{d}/t5child/c.rs").read() if r.returncode == 0 else ""
base_text = open(f"{src}/gate_table_c.rs").read()
want_edge = ("/// One documented hierarchy edge, from `hierarchy-edges.txt`.\npub(crate) struct Edge {\n    pub(crate) child: String,\n"
             "    pub(crate) parent: String,\n    pub(crate) line: String,\n}")
edge_ok = want_edge in child
fn_ok = "\npub(super) fn read_edges() -> Vec<Edge> {\n" in child
a = base_text.index("/// Reads one committed row file")
b = base_text.index("\n}\n", a) + 2
table_ok = base_text[a:b] in child
stripped = re.sub(r"pub\((crate|super)\) ", "", child)
units_ok = all(base_text.count(x) == 1 for x in [stripped[stripped.index("/// One documented"):]])
report("T5 move_items10.py widens exactly the declared declarations and fields",
       r.returncode == 0 and edge_ok and fn_ok and table_ok and units_ok,
       f"exit {r.returncode}; Edge {edge_ok}, read_edges {fn_ok}, read_table verbatim {table_ok}, "
       f"the child's units verbatim in BASE once the widening is removed {units_ok}\n{r.stdout}{r.stderr}")

# ------------------------------------------------------------------ part B
same = lambda o: o
drop_opt = lambda prefix: (lambda o: [x for x in o if not x.startswith(prefix)])
PART_B = {
    1: [
        ("B1-c1 a moved fn's body changes a token", ["I1b", "I2"], "table_c/rows.rs",
         first("status: row[4].clone(),", "status: row[5].clone(),"), same),
        ("B2-c1 a moved panic message changes", ["I1b", "I3"], "table_c/rows.rs",
         first('panic!("cannot read {}: {e}", path.display())', 'panic!("cannot read {}: {e}.", path.display())'), same),
        ("B3-c1 a moved row reader is deleted", ["I2"], "table_c/rows.rs",
         delete_item("pub(crate) fn read_edges()"), same),
        ("B4-c1 an unmoved line of gate_table_c.rs changes", ["I1a"], "gate_table_c.rs",
         first('const WIRING_PHASE: &str = "5a";', 'const WIRING_PHASE: &str = "5b";'), same),
        ("B5-c1 two moved row types swap their doc comments", ["I1b", "I2", "I3"], "table_c/rows.rs",
         swap("/// One `provide.xml` section, from `provide-sections.txt`.", "/// One documented hierarchy edge, from `hierarchy-edges.txt`."), same),
        ("B6-c1 the removed import left undeclared", ["I1a"], None, None, drop_opt("--expect-gone=")),
        ("B7-c1 two fields of a moved struct swap", ["I1b", "I2"], "table_c/rows.rs",
         first("    pub(crate) child: String,\n    pub(crate) parent: String,\n", "    pub(crate) parent: String,\n    pub(crate) child: String,\n"), same),
    ],
    2: [
        ("B1-c2 a moved probe line's literal changes", ["I1b", "I3"], "table_c/probes.rs",
         first('text.push_str("edge = 0\\n");', 'text.push_str("edge = 1\\n");'), same),
        # Whitespace-only to instrument 1 (b), which admits a reflow and defers
        # to 2 and 3, and invisible to whitespace-stripped instrument 2: the
        # decoded value is the only instrument that can see it (first run
        # expected I1b as well; corrected).
        ("B2-c2 a space inside a continued literal, seen by instrument 3 alone", ["I3"], "table_c/probes.rs",
         first("\\x20  renders as and what its class is", "\\x20 renders as and what its class is"), same),
        ("B3-c2 a moved helper is deleted", ["I2"], "table_c/probes.rs",
         delete_item("pub(crate) fn derived_say_lines("), same),
        ("B4-c2 two moved constants swap their values", ["I1b", "I3"], "table_c/probes.rs",
         swap('"gate-tables/classes"', '"gate-tables/hierarchy"'), same),
        ("B5-c2 an unmoved line of gate_table_c.rs changes", ["I1a"], "gate_table_c.rs",
         first('const INSTANCE_ARM: &str = "instance";', 'const INSTANCE_ARM: &str = "instances";'), same),
        ("B6-c2 an existing destination (mod.rs) changes a line", ["I1c"], "table_c/mod.rs",
         first("pub(super) mod rows;", "pub(crate) mod rows;"), same),
    ],
}
for n, plants in PART_B.items():
    p = pair(n)
    if p is None:
        print(f"part B c{n}: no commit yet, skipped")
        continue
    if pre_mode and n != int(sys.argv[2]):
        continue
    label, pre_t, post_t, removed, args, where = p
    env = env_for(args, where)
    opts = args[2:]
    rc, tags, out = instruments(here, pre_t, post_t, removed, env, opts)
    report(f"B c{n} as committed passes ({label})", rc == 0, f"exit {rc}, reported {tags}")
    for name, expect, rel, fn, optfn in plants:
        d = f"{work}/b-c{n}-{name.split()[0]}"
        shutil.rmtree(d, ignore_errors=True)
        shutil.copytree(post_t, d)
        if rel is not None:
            q = os.path.join(d, rel)
            s = open(q).read()
            s2 = fn(s)
            assert s2 != s, name
            open(q, "w").write(s2)
        o2 = optfn(opts)
        assert rel is not None or o2 != opts, name
        rc, tags, out = instruments(here, pre_t, d, removed, env, o2)
        ok = rc == 1 and set(expect) <= set(tags)
        report(f"{name} ({label})", ok, f"exit {rc}, expected {expect}, reported {tags}\n" + "\n".join(l for l in out.splitlines() if l[:2] in ("I1", "I2", "I3"))[:3000])

print(f"{total - missed} of {total} controls caught as expected")
sys.exit(1 if missed else 0)
