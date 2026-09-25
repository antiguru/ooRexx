"""Negative controls for this task's own tooling changes, on its own commit
pairs extracted with `git archive`: copy the POST tree, plant a defect a pure
move must not contain, and show each is reported. Each is planted in its own
copy.

  1. c8 (hash/stem.rs): a moved row of a `const` table, which item-tool now
     rows, changes its arity                                  -> I1b, I2, I3
  2. c8: a moved row is dropped from the child's table        -> I2
  3. c9 (hash/relation.rs): an unmoved row of hash.rs's `const` table points
     at another function                                      -> I1a, I2
  4. c7 (method_arguments.rs): a declared module-doc line carries one word
     more than was declared                                   -> I1a
  5. c13 (array/sort.rs): a token changes inside `slots_of`, an unmoved
     unit whose visibility the commit widened and rustfmt re-wrapped
                                                              -> I1a, I2
  6. c14 (array/surface.rs): an unmoved comment line of collection.rs is
     deleted beside the declared deletions                     -> I1a
  7. c6 (Directory's natives into hash.rs), with --expect-drop=hash::: a
     declared unit changes a token beyond the dropped qualifiers -> I2
  8. c6, with --expect-drop=hash::: an unmoved line in dispatch.rs renames
     a function                                               -> I1a, I2

A control passes when the checker exits 1 and reports at least the
instruments listed.

usage: controls_task6.py   (trees and removed.json located by commit)
"""
import os, shutil, subprocess, sys
S = "/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-6"
M = "/home/moritz/dev/repos/ooRexx-rust-rewrite"
REC = f"{M}/docs/superpowers/records/2026-09-15-file-split/task-6-files"
here = os.path.dirname(os.path.abspath(__file__))


def tree(c):
    d = f"{S}/trees/{c}"
    if not os.path.isdir(d):
        os.makedirs(d)
        a = subprocess.run(["git", "-C", M, "archive", c, "rust/crates/rexx-exec/src"], capture_output=True, check=True).stdout
        subprocess.run(["tar", "-x", "-C", d], input=a, check=True)
    return f"{d}/rust/crates/rexx-exec/src"


def parent(c):
    return subprocess.run(["git", "-C", M, "rev-parse", "--short=9", c + "^"], capture_output=True, text=True, check=True).stdout.strip()


def commit(n):
    """The commit whose message cites task-6-files/c<n>/."""
    out = subprocess.run(["git", "-C", M, "log", "--format=%h", f"--grep=task-6-files/c{n}/", "-F", "ed8cb3f03..HEAD"], capture_output=True, text=True, check=True).stdout.split()
    assert len(out) == 1, (n, out)
    return out[0]


EDIT6 = ["--expect-edit=fn native_hash_at", "--expect-edit=fn native_hash_put", "--expect-edit=fn native_hash_unknown", "--expect-drop=hash::"]
GONE14 = ["--expect-gone=", "--expect-gone=/// The collection classes' primitive methods, chained into", "--expect-gone=/// `ObjectModel::build`.", "--expect-gone=pub(super) const NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[", "--expect-gone=];"]
CASES = [
    (8, "dispatch/hash.rs", "dispatch/hash/stem.rs", "none", [], [
        ("1 moved const row changes its arity", ["I1b", "I2", "I3"], "dispatch/hash/stem.rs",
         '("Stem", "ITEMS", Arity::Fixed(0), native_stem_items)', '("Stem", "ITEMS", Arity::Fixed(1), native_stem_items)'),
        ("2 moved row dropped", ["I2"], "dispatch/hash/stem.rs",
         '    ("Stem", "ITEMS", Arity::Fixed(0), native_stem_items),\n', ''),
    ]),
    (9, "dispatch/hash.rs", "dispatch/hash/relation.rs", "none", [], [
        ("3 unmoved const row repointed", ["I1a", "I2"], "dispatch/hash.rs",
         '("Table", "ITEMS", Arity::Fixed(0), native_hash_items)', '("Table", "ITEMS", Arity::Fixed(0), native_hash_empty)'),
    ]),
    (7, "dispatch/buffer.rs", "dispatch/method_arguments.rs", "none",
     ["--expect-line=//! `MutableBuffer`'s primitive methods, and the argument and byte-search", "--expect-line=//! helpers the other primitive methods share."], [
        ("4 declared doc line carries more", ["I1a"], "dispatch/buffer.rs",
         "//! helpers the other primitive methods share.", "//! helpers the other primitive methods all share."),
    ]),
    (13, "dispatch/collection.rs", "dispatch/array/sort.rs", "none", [], [
        ("5 widened reflowed unit changes a token", ["I1a", "I2"], "dispatch/collection.rs",
         "    let store = store_of(interp, receiver)?;\n    array_slots_owned(interp, store)", "    let store = store_of(interp, receiver)?;\n    array_slots_owned(interp, receiver)"),
    ]),
    (14, "dispatch/collection.rs", "dispatch/array/surface.rs", "none", GONE14, [
        ("6 undeclared deletion beside the declared ones", ["I1a"], "dispatch/collection.rs",
         "// ---- the contents protocol ----\n", ""),
    ]),
    (6, "dispatch.rs", "dispatch/hash.rs", "none", EDIT6, [
        ("7 declared unit changes beyond the qualifiers", ["I2"], "dispatch/hash.rs",
         "let index = hash_index(interp, args, 1)?;", "let index = hash_index(interp, args, 2)?;"),
        ("8 unmoved dispatch.rs line changes", ["I1a", "I2"], "dispatch.rs",
         "fn put_native(", "fn put_native_(",),
    ]),
]
work = f"{S}/ctl-work/task6"
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
        if old is None:
            # Delete the last `#[test]` fn of the file, doc and all.
            i = s.rindex("\n#[test]\n")
            j = s.rindex("\n\n", 0, i) + 1
            s = s[:j] + s[s.index("\n}\n", i) + 3:]
        else:
            assert s.count(old) >= 1, (rel, old)
            i = s.find(old); s = s[:i] + new + s[i + len(old):]
        open(p, "w").write(s)
        r = subprocess.run([sys.executable, os.path.join(here, "instruments.py"), pre, d, removed, d + "-out"] + extra, capture_output=True, text=True, env=env)
        seen = [tag for tag in expect if any(line.startswith(tag + " ") for line in r.stdout.splitlines())]
        ok = r.returncode == 1 and seen == expect
        missed += not ok
        print(f"control {name} (c{n} {post_c}): exit {r.returncode}, expected {expect}, reported {seen}: {'CAUGHT' if ok else 'MISSED'}")
        print("  " + (r.stdout + r.stderr).strip().replace("\n", "\n  "))
print(f"{total - missed} of {total} controls caught as expected")
sys.exit(1 if missed else 0)
