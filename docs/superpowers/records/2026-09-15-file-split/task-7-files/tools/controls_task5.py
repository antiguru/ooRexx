"""Negative controls for this task's own tooling changes, on its own commit
pairs extracted with `git archive`: copy the POST tree, plant a defect a pure
move must not contain, and show each is reported. Each is planted in its own
copy.

  1. c2 (plan.rs tests): a moved unit rustfmt re-wrapped to another line
     count (the case instruments.py no longer re-indents) changes a token
                                                              -> I1b, I2
  2. c5 (object_operand_tests): a moved test vanishes from the child, which
     is found only if the child's keys carry `object_operand_tests::`
                                                              -> I2
  3. c5: a space inside a moved test's string literal: whitespace-only to
     instruments 1 (b) and 2 by design, so instrument 3 is the one that
     fails                                                    -> I3
  4. c4 (eval tests, one declared edit): a test other than the declared one
     changes a token, so the declaration masks nothing else   -> I1b, I2
  5. c7 (version.rs): a moved const's literal changes       -> I1b, I2, I3
  6. c9 (environment/route.rs): an unmoved line of `ORACLE_LOCAL`, a
     static list of names that item-tool now reports as one unit, changes
     a name                                                  -> I1a, I2, I3
  7. c9: a moved member changes a token                       -> I1b, I2

usage: controls_task5.py   (trees and removed.json located by commit)
"""
import os, shutil, subprocess, sys
S = "/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-7"
M = "/home/moritz/dev/repos/ooRexx-rust-rewrite"
REC = f"{M}/docs/superpowers/records/2026-09-15-file-split/task-5-files"
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
    """The commit whose message cites task-5-files/c<n>/."""
    out = subprocess.run(["git", "-C", M, "log", "--format=%h", f"--grep=task-5-files/c{n}/", "-F"], capture_output=True, text=True, check=True).stdout.split()
    assert len(out) == 1, (n, out)
    return out[0]


CASES = [
    (2, "plan.rs", "plan/tests.rs", "plan/tests.rs", [], [
        ("1 re-wrapped unit changes a token", ["I1b", "I2"], "plan/tests.rs",
         "fn line_at_answers_what_line_of_answers_at_every_index() {", "fn line_at_answers_what_line_of_answers_at_every_index() {\n    let _ = 0;"),
    ]),
    (5, "eval.rs", "eval/object_operand_tests.rs", "eval/object_operand_tests.rs", [], [
        ("2 moved test vanishes from the child", ["I2"], "eval/object_operand_tests.rs", None, "fn "),
        ("3 space inside a moved test's literal", ["I3"], "eval/object_operand_tests.rs", "(.Object~superClasses & 1)", "(.Object~superClasses &  1)"),
    ]),
    (4, "eval.rs", "eval/tests.rs", "eval/tests.rs", ["--expect-edit=tests::fn the_small_int_fast_path_answers_what_the_general_path_answers"], [
        ("4 undeclared test changes a token", ["I1b", "I2"], "eval/tests.rs", "interp.to_text(value).to_vec()", "interp.to_text(value).to_owned()"),
    ]),
    (7, "parse_template.rs", "version.rs", "none", [], [
        ("5 moved const changes its literal", ["I1b", "I2", "I3"], "version.rs", 'b"LINUX"', 'b"LINUS"'),
    ]),
    (9, "environment.rs", "environment/route.rs", "none", [], [
        ("6 unmoved list static changes a name", ["I1a", "I2", "I3"], "environment.rs", '    "DEBUGINPUT",\n', '    "DEBUG_INPUT",\n'),
        ("7 moved member changes a token", ["I1b", "I2"], "environment/route.rs",
         "self.route_generation.wrapping_add(1)", "self.route_generation.wrapping_add(2)"),
    ]),
]
work = f"{S}/ctl-work/task5"
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
