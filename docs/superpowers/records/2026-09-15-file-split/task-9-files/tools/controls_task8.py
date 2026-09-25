"""Negative controls for this task's tooling changes (part A, on Task 7's
commit pairs, runnable before this task moves anything) and for its own
commits (part B, run once they exist). Trees come from `git archive`; each
plant is made in its own copy.

Part A, the two gaps Task 7's review found, and the unsafe count:

  A1. Task 7's c4 with --expect-reflow, and the declared unit's argument
      turned from `(x)` into `(x,)`, a parenthesised expression into a
      1-tuple (review-7's `ctl-paren`, planted on both sides)
                                             -> exit 1, I1b and I2
      and Task 7's own instruments.py on the same trees (the tool before
      the fix) passes it, which is the gap.
  A2. Task 7's c4 as committed with --expect-reflow still passes: the
      restriction keeps the comma before `]` the unit really lost.
  A3. comments7.py on Task 7's c6: a positional comment planted in the
      parent (`drive.rs`) that names no moved unit is listed under (c);
      Task 7's own comments7.py on the same tree does not list it.
  A4. unsafe_count.sh on Task 7's c5: `unsafe {` planted in the
      destination's code is FOUND; the same text in a `//` comment is not;
      and `unsafe fn` planted in PRE's moved lines is FOUND.

Part B, instruments 1-3 on this task's own commits (a control passes when
instruments.py exits 1 and reports at least the instruments listed):

  B1. c1 (invoke tests): a string literal inside a moved test's macro
      arguments changes                      -> I1b, I2, I3
  B2. c2 (values/convert.rs): a moved converter's body changes a token
                                             -> I1b, I2
  B3. c2: a moved converter is deleted       -> I2
  B4. c2: an unmoved line of values.rs (a TABLE row) changes -> I1a
  B5. c2: two moved converters swap their doc comments, so each doc sits
      on the other's item                    -> I1b, I2, I3
  B6. c2: the `reason = "..."` string inside a moved converter's
      `#[expect(..)]` attribute changes      -> I1b, I2, I3
  B9. c2: the string a moved converter passes to `.expect(..)` changes
                                             -> I1b, I2, I3
  B7. c1 as committed but without its --expect-reflow (the test module's
      `use crate::ffi::{..}`, which rustfmt joined onto one line, dropping
      the comma before `}`)                  -> I1b, I2
  B8. c1: that reflowed import also loses a name  -> I2

usage: controls_task8.py
"""
import os, shutil, subprocess, sys
os.environ["PYTHONDONTWRITEBYTECODE"] = "1"
S = "/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-9"
M = "/home/moritz/dev/repos/ooRexx-rust-rewrite"
REC = f"{M}/docs/superpowers/records/2026-09-15-file-split"
here = os.path.dirname(os.path.abspath(__file__))
T7 = f"{REC}/task-7-files/tools"
work = f"{S}/ctl-work/task8"
# Task 7's committed tools run with this task's item-tool build (its source
# is Task 7's, unchanged); the records hold no build of it.
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
    for r, o, n in ([(rel, old, new)] if isinstance(rel, str) else rel):
        if o is None:
            continue
        p = os.path.join(dst, r)
        s = open(p).read()
        assert s.count(o) == 1, (r, o, s.count(o))
        open(p, "w").write(s.replace(o, n))
    return dst


def instruments(tool_dir, pre, post, removed, env, extra):
    r = subprocess.run([sys.executable, os.path.join(tool_dir, "instruments.py"), pre, post, removed, post + "-out"] + extra, capture_output=True, text=True, env=dict(env, ITEM_TOOL=ITEM_TOOL))
    tags = sorted({l.split(" ")[0] for l in r.stdout.splitlines() if l[:2] in ("I1", "I2", "I3")})
    return r.returncode, tags, (r.stdout + r.stderr).strip()


# ------------------------------------------------------------------ part A
T7BASE = "964a6a8db"
c4 = commit(7, 4, T7BASE)
env4 = dict(os.environ, SPLIT_PARENT="ir/compile", SPLIT_PARENT_RS="ir/compile.rs", SPLIT_DESTS="ir/compile/tests.rs", SPLIT_TESTS_RS="ir/compile/tests.rs")
extra4 = ["--tests", "--expect-reflow=tests::fn a_trace_op_outside_a_clause_region_is_refused"]
rem4 = f"{REC}/task-7-files/c4/removed.json"
pre = plant(tree(parent(c4)), f"{work}/a1-pre", "ir/compile.rs",
            "        assert_trace_ops_open_a_clause_region(&[\n            Op::Exec { index: 0 },\n            Op::TraceClause { index: 1 },\n        ]);\n",
            "        assert_trace_ops_open_a_clause_region((&[\n            Op::Exec { index: 0 },\n            Op::TraceClause { index: 1 },\n        ]));\n")
post = plant(tree(c4), f"{work}/a1-post", "ir/compile/tests.rs",
             "    assert_trace_ops_open_a_clause_region(&[Op::Exec { index: 0 }, Op::TraceClause { index: 1 }]);\n",
             "    assert_trace_ops_open_a_clause_region((&[Op::Exec { index: 0 }, Op::TraceClause { index: 1 }],));\n")
rc, tags, out = instruments(here, pre, post, rem4, env4, extra4)
rc7, tags7, _ = instruments(T7, pre, post, rem4, env4, extra4)
report("A1 --expect-reflow: (x) becomes (x,) in the declared unit", rc == 1 and {"I1b", "I2"} <= set(tags) and rc7 == 0,
       f"this task's instruments.py: exit {rc}, reported {tags}; Task 7's: exit {rc7}, reported {tags7}\n" + "\n".join(l for l in out.splitlines() if l.startswith("I")))
rc, tags, out = instruments(here, tree(parent(c4)), tree(c4), rem4, env4, extra4)
report("A2 --expect-reflow: Task 7's c4 as committed still passes", rc == 0, f"exit {rc}, reported {tags}\n" + out.splitlines()[0])

c6 = commit(7, 6, T7BASE)
d = plant(tree(c6, "rust"), f"{work}/a3", "crates/rexx-exec/src/ir/drive.rs",
          "\nfn debug_assert_names_the_clause(", "\n// Everything below this line is planted, and names no moved unit.\nfn debug_assert_names_the_clause(")
args = [tree(parent(c6)), d, "ir/drive.rs", "ir/counters.rs"]
cenv = dict(os.environ, ITEM_TOOL=ITEM_TOOL)
new = subprocess.run([sys.executable, os.path.join(here, "comments7.py")] + args, capture_output=True, text=True, env=cenv).stdout
old = subprocess.run([sys.executable, os.path.join(T7, "comments7.py")] + args, capture_output=True, text=True, env=cenv).stdout
hit = lambda o: [l for l in o.splitlines() if "is planted" in l]
ran = lambda o: [l for l in o.splitlines() if l.startswith("(c) ")]
report("A3 comments7: a positional comment in the parent naming no moved unit", len(hit(new)) == 1 and "drive.rs" in hit(new)[0] and hit(old) == [] and ran(old) != [],
       f"this task's comments7.py lists: {hit(new)} under {ran(new)}\nTask 7's lists: {hit(old)} under {ran(old)}")

c5 = commit(7, 5, T7BASE)
rem5 = f"{REC}/task-7-files/c5/removed.json"
uc = lambda pre, post: subprocess.run([os.path.join(here, "unsafe_count.sh"), pre, post, rem5, "ir/compile.rs", "ir/compile/invariants.rs"], capture_output=True, text=True).stdout
base_out = uc(tree(parent(c5)), tree(c5))
code = plant(tree(c5), f"{work}/a4-code", "ir/compile/invariants.rs", "\npub(super) fn assert_region_ops_name_their_clause(", "\npub(super) fn _planted() { unsafe {} }\npub(super) fn assert_region_ops_name_their_clause(")
comment = plant(tree(c5), f"{work}/a4-comment", "ir/compile/invariants.rs", "\npub(super) fn assert_region_ops_name_their_clause(", "\n// unsafe { is planted in a comment }\npub(super) fn assert_region_ops_name_their_clause(")
prep = plant(tree(parent(c5)), f"{work}/a4-pre", "ir/compile.rs", "\nfn assert_trace_ops_open_a_clause_region(", "\nunsafe fn assert_trace_ops_open_a_clause_region(")
o_code, o_comment, o_pre = uc(tree(parent(c5)), code), uc(tree(parent(c5)), comment), uc(prep, tree(c5))
last = lambda o: o.strip().splitlines()[-1]
report("A4 unsafe_count: code, comment, moved PRE lines",
       last(base_out) == "unsafe: NONE" and last(o_code).startswith("unsafe: FOUND") and last(o_comment) == "unsafe: NONE" and last(o_pre).startswith("unsafe: FOUND"),
       f"as committed: {last(base_out)}\nplanted in code: {last(o_code)}\nplanted in a comment: {last(o_comment)}\nplanted in PRE's moved lines: {last(o_pre)}")

# ------------------------------------------------------------------ part B
BASE = "5c7173fac"
REFLOW1 = "--expect-reflow=tests::use crate::ffi::{CALL_CONTEXT,CallContext,MethodContext,Seen,forget_seen,reading_stub,seen}"
CASES = [
    (1, "invoke.rs", "invoke/tests.rs", "invoke/tests.rs", [REFLOW1], []),
    (2, "values.rs", "values/convert.rs", "none", [], []),
]
B = {
    1: [("B1 a literal inside a moved test's macro arguments", ["I1b", "I2", "I3"], "invoke/tests.rs", "LIT1", None),
        ("B7 c1 without its declared reflow", ["I1b", "I2"], "invoke/tests.rs", "NOREFLOW", None),
        ("B8 the reflowed import also loses a name", ["I2"], "invoke/tests.rs", "reading_stub, seen};", "reading_stub};")],
    2: [
        ("B2 a moved converter's body changes a token", ["I1b", "I2"], "values/convert.rs",
         "    let (min, max) = (i64::from(i8::MIN), i64::from(i8::MAX));\n", "    let (min, max) = (i64::from(i8::MIN), i64::from(i8::MIN));\n"),
        ("B3 a moved converter is deleted", ["I2"], "values/convert.rs", "DELETE", None),
        ("B4 an unmoved TABLE row of values.rs changes", ["I1a"], "values.rs",
         "    argument(code::INT, \"int\", Repr::Int, int_to_native, int_from_native),\n",
         "    argument(code::INT, \"int\", Repr::Int, int_to_native, int8_from_native),\n"),
        ("B5 two moved converters swap their doc comments", ["I1b", "I2", "I3"], "values/convert.rs", "SWAPDOC", None),
        ("B6 a string literal inside a moved converter's attribute changes", ["I1b", "I2", "I3"], "values/convert.rs",
         "which is the conversion measured\"", "which is the conversion measured.\""),
        ("B9 a string literal in a moved converter's method-call argument changes", ["I1b", "I2", "I3"], "values/convert.rs",
         ".expect(\"a c_int is a small integer\")", ".expect(\"a c_int is a small integer.\")"),
    ],
}
for n, parent_rs, dests, tests_rs, opts, _ in CASES:
    c = commit(8, n, BASE)
    if c is None:
        print(f"part B c{n}: no commit yet, skipped")
        continue
    pre_t, post_t = tree(parent(c), "rust/crates/rexx-api/src"), tree(c, "rust/crates/rexx-api/src")
    removed = f"{REC}/task-8-files/c{n}/removed.json"
    env = dict(os.environ, SPLIT_CRATE="rexx-api", SPLIT_PARENT=parent_rs[:-3], SPLIT_PARENT_RS=parent_rs, SPLIT_DESTS=dests, SPLIT_TESTS_RS=tests_rs)
    extra = (["--tests"] if tests_rs != "none" else []) + opts
    rc, tags, out = instruments(here, pre_t, post_t, removed, env, extra)
    report(f"B c{n} as committed passes", rc == 0, f"exit {rc}, reported {tags}")
    for name, expect, rel, old, new in B[n]:
        d = f"{work}/b-c{n}-{name.split()[0]}"
        shutil.rmtree(d, ignore_errors=True)
        shutil.copytree(post_t, d)
        p = os.path.join(d, rel)
        s = open(p).read()
        if old in ("LIT1", "LIT2"):
            import re
            pat = r'assert_eq!\([^;]*?"([A-Za-z]{3,})"' if old == "LIT1" else r"b'([a-z()])'"
            m = re.search(pat, s, re.S)
            assert m, (rel, pat)
            a, b = m.span(1)
            s = s[:a] + ("X" + s[a + 1:b] if old == "LIT1" else ("q" if s[a:b] != "q" else "z")) + s[b:]
        elif old == "NOREFLOW":
            pass
        elif old == "DELETE":
            a = s.index("/// `REXX_VALUE_int16_t`")
            b = s.index("\n}\n", a) + 3
            s = s[:a] + s[b + 1:]
        elif old == "SWAPDOC":
            x, y = "/// `REXX_VALUE_int8_t`", "/// `REXX_VALUE_int16_t`"
            assert s.count(x) == 1 and s.count(y) == 1
            s = s.replace(x, "\0").replace(y, x).replace("\0", y)
        else:
            assert s.count(old) == 1, (rel, old, s.count(old))
            s = s.replace(old, new)
        open(p, "w").write(s)
        rc, tags, out = instruments(here, pre_t, d, removed, env, [x for x in extra if x not in opts] if old == "NOREFLOW" else extra)
        ok = rc == 1 and set(expect) <= set(tags)
        report(f"{name} (c{n} {c})", ok, f"exit {rc}, expected {expect}, reported {tags}\n" + "\n".join(l for l in out.splitlines() if l[:2] in ("I1", "I2", "I3"))[:3000])

print(f"{total - missed} of {total} controls caught as expected")
sys.exit(1 if missed else 0)
