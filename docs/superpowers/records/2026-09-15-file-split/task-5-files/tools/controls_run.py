"""Negative controls for instruments 1-3 on this task's own last commit pair
(run.rs before and after the loops move): copy the POST tree, plant a defect
a pure move must not contain, and show each is reported.

  1. an unmoved run.rs line gains a space                             -> I1a
  2. a moved string literal inside a macro argument gains a space
     (`HeaderPlan::push`'s `debug_assert!` message), token-blind        -> I3
  3. a moved method's token changes (`self.len += 1` -> `+= 2`)         -> I1b, I2
  4. a moved re-wrapped signature changes a token next to the comma the
     relaxation drops (`numeric_less`'s `fuzz: u64,` -> `fuzz: u32,`)   -> I1b, I2
Each is planted in its own copy.

usage: controls_run.py PRE_ROOT POST_ROOT REMOVED_JSON WORKDIR
"""
import os, shutil, subprocess, sys
pre, post, removed, work = sys.argv[1:5]
here = os.path.dirname(os.path.abspath(__file__))
CONTROLS = [
    ("1 unmoved line gains a space", ["I1a"], "run.rs",
     "fn standard_stream_name(name: &[u8], input: bool)", "fn standard_stream_name(name: &[u8],  input: bool)"),
    ("2 space inside a macro argument's string", ["I3"], "run/loops.rs",
     "a DO/LOOP header has more", "a DO/LOOP  header has more"),
    ("3 token change", ["I1b", "I2"], "run/loops.rs", "self.len += 1;", "self.len += 2;"),
    ("4 token change beside a re-wrapped signature's comma", ["I1b", "I2"], "run/loops.rs",
     "    fuzz: u64,\n) -> Result<bool, ArithError>", "    fuzz: u32,\n) -> Result<bool, ArithError>"),
]
env = dict(os.environ, SPLIT_PARENT="run")
missed = 0
for name, expect, rel, old, new in CONTROLS:
    d = os.path.join(work, name.split()[0])
    shutil.rmtree(d, ignore_errors=True)
    os.makedirs(d)
    shutil.copy(os.path.join(post, "run.rs"), d)
    shutil.copytree(os.path.join(post, "run"), os.path.join(d, "run"))
    p = os.path.join(d, rel); s = open(p).read()
    assert s.count(old) == 1, (rel, old)
    open(p, "w").write(s.replace(old, new))
    r = subprocess.run([sys.executable, os.path.join(here, "instruments.py"), pre, d, removed, os.path.join(d, "out")], capture_output=True, text=True, env=env)
    seen = [tag for tag in expect if any(line.startswith(tag + " ") for line in r.stdout.splitlines())]
    ok = r.returncode == 1 and seen == expect
    missed += not ok
    print(f"control {name}: exit {r.returncode}, expected {expect}, reported {seen}: {'CAUGHT' if ok else 'MISSED'}")
    print("  " + (r.stdout + r.stderr).strip().replace("\n", "\n  "))
print(f"{len(CONTROLS) - missed} of {len(CONTROLS)} controls caught as expected")
sys.exit(1 if missed else 0)
