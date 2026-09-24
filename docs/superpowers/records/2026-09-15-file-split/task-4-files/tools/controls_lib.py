"""Negative controls for this task's own tooling changes, on its own commit
pairs: copy the POST tree, plant a defect a pure move must not contain, and
show each is reported. Each is planted in its own copy.

  1. c1: a line of a moved byte string that the mover keeps verbatim (it
     starts inside the literal) gains a space: `::class k` -> `::class  k`;
     token-blind and whitespace-only, so instrument 1 (b) lists it as
     whitespace-only and instrument 3 is the one that fails      -> I3
  2. c1: a moved test's de-indented code line keeps its value but changes
     a token (`TEST_PATH` const's path gains a character)      -> I1b, I2, I3
  3. c1: an import in the parent gains a name instead of losing one
                                                              -> I1a
  4. seal commit: an unmoved line of the existing destination run.rs gains a
     space                                                    -> I1c

usage: controls_lib.py CASE PRE_ROOT POST_ROOT REMOVED_JSON WORKDIR
  (CASE is c1 or seal; SPLIT_PARENT_RS / SPLIT_DESTS / SPLIT_TESTS_RS are
  set here per case)
"""
import os, shutil, subprocess, sys
case, pre, post, removed, work = sys.argv[1:6]
here = os.path.dirname(os.path.abspath(__file__))
CASES = {
    "c1": (dict(SPLIT_PARENT="lib", SPLIT_PARENT_RS="lib.rs", SPLIT_DESTS="tests.rs", SPLIT_TESTS_RS="tests.rs"), ["--tests"], [
        ("1 verbatim literal line gains a space", ["I3"], "tests.rs", "\n::class k\n", "\n::class  k\n"),
        ("2 de-indented line changes a literal", ["I1b", "I2", "I3"], "tests.rs",
         '"/nonexistent/lib-test-program.rex"', '"/nonexistent/lib-test-program.rexx"'),
        ("3 parent import gains a name", ["I1a"], "lib.rs", "use std::rc::Rc;\n", "use std::rc::{Rc, Weak};\n"),
    ]),
    "seal": (dict(SPLIT_PARENT="run/interpret", SPLIT_PARENT_RS="run/interpret.rs", SPLIT_DESTS="run.rs", SPLIT_TESTS_RS="none"), [], [
        ("4 unmoved destination line gains a space", ["I1c"], "run.rs",
         "fn standard_stream_name(name: &[u8], input: bool)", "fn standard_stream_name(name: &[u8],  input: bool)"),
    ]),
}
env_extra, opts, controls = CASES[case]
env = dict(os.environ, **env_extra)
missed = 0
for name, expect, rel, old, new in controls:
    d = os.path.join(work, case + "-" + name.split()[0])
    shutil.rmtree(d, ignore_errors=True)
    shutil.copytree(post, d)
    p = os.path.join(d, rel); s = open(p).read()
    assert s.count(old) == 1, (rel, old, s.count(old))
    open(p, "w").write(s.replace(old, new))
    r = subprocess.run([sys.executable, os.path.join(here, "instruments.py"), pre, d, removed, os.path.join(work, case + "-" + name.split()[0] + "-out")] + opts, capture_output=True, text=True, env=env)
    seen = [tag for tag in expect if any(line.startswith(tag + " ") for line in r.stdout.splitlines())]
    ok = r.returncode == 1 and seen == expect
    missed += not ok
    print(f"control {name}: exit {r.returncode}, expected {expect}, reported {seen}: {'CAUGHT' if ok else 'MISSED'}")
    print("  " + (r.stdout + r.stderr).strip().replace("\n", "\n  "))
print(f"{len(controls) - missed} of {len(controls)} controls caught as expected")
sys.exit(1 if missed else 0)
