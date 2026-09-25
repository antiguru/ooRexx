"""Negative controls for this task's own tooling changes, on its own commit
pairs extracted with `git archive`: copy the POST tree, plant a defect a pure
move must not contain, and show each is reported. Each is planted in its own
copy.

Instruments 1-3 (a control passes when instruments.py exits 1 and reports
at least the instruments listed):

  1. c6 (counters.rs): a moved `thread_local!`, which item-tool now keys by
     the static it declares, changes its initial value  -> I1b, I2, I3
  2. c6: a moved `thread_local!` is deleted                -> I2
  3. c6: two moved `thread_local!` blocks swap the statics they declare,
     so each key's tokens are unchanged but its comment is not -> I1b
  4. c4 (compile tests), with --expect-reflow: the declared unit also
     changes a token                                        -> I1b, I2, I3
  5. c4 as committed but without --expect-reflow: the declaration is what
     admits the reflow                                      -> I1b, I2
  6. c5 (invariants): an unmoved line of compile.rs changes -> I1a

The other tools (a control passes when the tool reports the plant):

  7. docsig.py: a warning that moves to another file changes the
     signature; the same warning at another line does not
  8. comments7.py: a comment elsewhere naming a moved counter through the
     parent's module (`drive::run_chunk_entries`) is listed
  9. other_edits.py on c5's rule: a second edit in `drive.rs` beside the
     declared substitution fails the check
 10. other_edits.py on c6's rule: an edited (not inserted) line in `ir.rs`
     fails the check

usage: controls_task7.py   (trees and removed.json located by commit)
"""
import os, shutil, subprocess, sys
S = "/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-10"
M = "/home/moritz/dev/repos/ooRexx-rust-rewrite"
REC = f"{M}/docs/superpowers/records/2026-09-15-file-split/task-7-files"
BASE = "964a6a8db"
here = os.path.dirname(os.path.abspath(__file__))


def tree(c, what="rust/crates/rexx-exec/src"):
    d = f"{S}/trees/{c}"
    if not os.path.isdir(d):
        os.makedirs(d)
        a = subprocess.run(["git", "-C", M, "archive", c, "rust"], capture_output=True, check=True).stdout
        subprocess.run(["tar", "-x", "-C", d], input=a, check=True)
    return f"{d}/{what}"


def parent(c):
    return subprocess.run(["git", "-C", M, "rev-parse", "--short=9", c + "^"], capture_output=True, text=True, check=True).stdout.strip()


def commit(n):
    """The commit whose message cites task-7-files/c<n>/."""
    out = subprocess.run(["git", "-C", M, "log", "--format=%h", f"--grep=task-7-files/c{n}/", "-F", f"{BASE}..HEAD"], capture_output=True, text=True, check=True).stdout.split()
    assert len(out) == 1, (n, out)
    return out[0]


REFLOW4 = ["--expect-reflow=tests::fn a_trace_op_outside_a_clause_region_is_refused"]
CASES = [
    (6, "ir/drive.rs", "ir/counters.rs", "none", [], [
        ("1 moved thread_local changes its value", ["I1b", "I2", "I3"], "ir/counters.rs",
         "static CLAUSE_OP_ENTRIES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };",
         "static CLAUSE_OP_ENTRIES: std::cell::Cell<usize> = const { std::cell::Cell::new(1) };"),
        ("2 moved thread_local deleted", ["I2"], "ir/counters.rs",
         "#[cfg(test)]\nthread_local! {\n    static CALL_SITE_HITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };\n}\n", ""),
        ("3 two moved thread_locals swap their statics", ["I1b"], "ir/counters.rs", "SWAP", None),
    ]),
    (4, "ir/compile.rs", "ir/compile/tests.rs", "ir/compile/tests.rs", REFLOW4, [
        ("4 declared reflow unit changes a token", ["I1b", "I2", "I3"], "ir/compile/tests.rs",
         "Op::TraceClause { index: 1 }]);", "Op::TraceClause { index: 2 }]);"),
    ]),
    (4, "ir/compile.rs", "ir/compile/tests.rs", "ir/compile/tests.rs", [], [
        ("5 the reflow undeclared", ["I1b", "I2"], "ir/compile/tests.rs", "", ""),
    ]),
    (5, "ir/compile.rs", "ir/compile/invariants.rs", "none", [], [
        ("6 unmoved compile.rs line changes", ["I1a"], "ir/compile.rs",
         "fn op_index(ops: &[Op])", "fn op_index_(ops: &[Op])"),
    ]),
]
work = f"{S}/ctl-work/task7"
missed = total = 0
for n, parent_rs, dests, tests_rs, opts, controls in CASES:
    post_c = commit(n)
    pre, post = tree(parent(post_c)), tree(post_c)
    removed = f"{REC}/c{n}/removed.json"
    env = dict(os.environ, SPLIT_PARENT=parent_rs[:-3], SPLIT_PARENT_RS=parent_rs, SPLIT_DESTS=dests, SPLIT_TESTS_RS=tests_rs)
    extra = (["--tests"] if tests_rs != "none" else []) + opts
    for name, expect, rel, old, new in controls:
        total += 1
        d = f"{work}/c{n}-{name.split()[0]}"
        shutil.rmtree(d, ignore_errors=True)
        shutil.copytree(post, d)
        p = os.path.join(d, rel); s = open(p).read()
        if old == "SWAP":
            a, b = "static RUN_CHUNK_ENTRIES:", "static CLAUSE_OP_ENTRIES:"
            assert s.count(a) == 1 and s.count(b) == 1
            s = s.replace(a, "\0").replace(b, a).replace("\0", b)
        elif old:
            assert s.count(old) >= 1, (rel, old)
            i = s.find(old); s = s[:i] + new + s[i + len(old):]
        open(p, "w").write(s)
        r = subprocess.run([sys.executable, os.path.join(here, "instruments.py"), pre, d, removed, d + "-out"] + extra, capture_output=True, text=True, env=env)
        seen = [tag for tag in expect if any(line.startswith(tag + " ") for line in r.stdout.splitlines())]
        ok = r.returncode == 1 and seen == expect
        missed += not ok
        print(f"control {name} (c{n} {post_c}): exit {r.returncode}, expected {expect}, reported {seen}: {'CAUGHT' if ok else 'MISSED'}")
        print("  " + (r.stdout + r.stderr).strip().replace("\n", "\n  "))


def report(name, ok, detail):
    global missed, total
    total += 1
    missed += not ok
    print(f"control {name}: {'CAUGHT' if ok else 'MISSED'}")
    print("  " + detail.strip().replace("\n", "\n  "))


# 7: docsig.py
os.makedirs(work, exist_ok=True)
log = "warning: unresolved link to `x`\n   --> crates/a.rs:10:5\n    |\nwarning: `c` (lib doc) generated 1 warning\n"
moved = log.replace("crates/a.rs:10:5", "crates/a/b.rs:10:5")
shifted = log.replace("crates/a.rs:10:5", "crates/a.rs:99:1")
sig = {}
for k, text in (("orig", log), ("moved", moved), ("shifted", shifted)):
    open(f"{work}/doc-{k}.txt", "w").write(text)
    sig[k] = subprocess.run([sys.executable, os.path.join(here, "docsig.py"), f"{work}/doc-{k}.txt"], capture_output=True, text=True, check=True).stdout
report("7 docsig: a warning moves file", sig["orig"] != sig["moved"] and sig["orig"] == sig["shifted"],
       f"orig:\n{sig['orig']}moved:\n{sig['moved']}shifted:\n{sig['shifted']}")

# 8: comments7.py
c6 = commit(6)
d = f"{work}/c6-comments"
shutil.rmtree(d, ignore_errors=True)
shutil.copytree(tree(c6, "rust"), d)
p = f"{d}/crates/rexx-exec/src/lib.rs"
s = open(p).read()
s = s.replace("        #[cfg(test)]\n        ir::drive::suspend_counters();\n", "        // `drive::run_chunk_entries` is planted.\n        #[cfg(test)]\n        ir::drive::suspend_counters();\n", 1)
open(p, "w").write(s)
r = subprocess.run([sys.executable, os.path.join(here, "comments7.py"), tree(parent(c6)), d, "ir/drive.rs", "ir/counters.rs"], capture_output=True, text=True)
report("8 comments7: a qualified mention elsewhere is listed", "lib.rs" in r.stdout and "is planted" in r.stdout,
       "\n".join(l for l in r.stdout.splitlines() if "planted" in l or l.startswith("(a)")))


# 9, 10: other_edits.py on this task's rules, in a scratch worktree at the
# commit's parent with the commit's files checked out over it.
W = f"{S}/oe7-wt"


def reset(c):
    if not os.path.isdir(W):
        subprocess.run(["git", "-C", M, "worktree", "add", "-q", "--detach", W, parent(c)], check=True)
    for cmd in (["checkout", "-q", "-f", parent(c)], ["clean", "-q", "-fd", "--", "rust/crates"], ["checkout", "-q", c, "--", "rust/crates"], ["reset", "-q"]):
        subprocess.run(["git", "-C", W] + cmd, check=True)


def oe(n):
    rules = [l for l in open(f"{REC}/c{n}/rules").read().split("\n") if l]
    r = subprocess.run([sys.executable, os.path.join(here, "other_edits.py"), f"{W}/rust", f"{work}/oe.out"] + rules, capture_output=True, text=True)
    return r.returncode, open(f"{work}/oe.out").read()


for n, name, rel, old, new in (
    (5, "9 other_edits: a second edit beside c5's substitution", "ir/drive.rs",
     "                    // is the same check per op in debug.\n", "                    // is the same check per op in debug builds.\n"),
    (6, "10 other_edits: an edited line in insert-only ir.rs", "ir.rs",
     "mod valid;\n", "pub(crate) mod valid;\n"),
):
    c = commit(n)
    reset(c)
    base_rc, base_out = oe(n)
    p = f"{W}/rust/crates/rexx-exec/src/{rel}"
    s = open(p).read()
    assert s.count(old) == 1, (rel, old)
    open(p, "w").write(s.replace(old, new))
    rc, out = oe(n)
    report(name, base_rc == 0 and rc == 1, f"as committed: exit {base_rc}; planted: exit {rc}\n" + "\n".join(l for l in out.splitlines() if "FAIL" in l))
subprocess.run(["git", "-C", M, "worktree", "remove", "--force", W], check=True)

print(f"{total - missed} of {total} controls caught as expected")
sys.exit(1 if missed else 0)
