"""Negative controls for instruments 1-3: copy commit 1's POST tree, plant
defects a pure move must not contain, and show each is reported.

  1. an unmoved line gains a space   (`const SWEEPS: usize = 2 ;`)      -> I1a
  2. a moved test's continued string gains a space inside its value
     (`\\x20 expose n` -> `\\x20  expose n`), token-blind by construction  -> I3
  3. a moved test's token changes    (first `ObjRef::NIL` -> `ObjRef::NUL`) -> I1b, I2
  4. a tuple loses its trailing comma in a moved fn's body, `(.., x,)` -> `(.., x)`
     (`run_source`'s returned tuple)                                     -> I1b, I2
  5. a tuple gains one, `(93, 903)` -> `(93, 903,)`                      -> I1b, I2
Each is planted in its own copy and run separately, so one control's report
cannot stand in for another's. The only comma instruments 1 (b) and 2 ignore is
the one closing a `fn` signature's parameter list, which 4 and 5 are not.

usage: controls.py PRE_ROOT POST_ROOT REMOVED_JSON WORKDIR
"""
import os, shutil, subprocess, sys
pre, post, removed, work = sys.argv[1:5]
here = os.path.dirname(os.path.abspath(__file__))
CONTROLS = [
    ("1 unmoved line gains a space", ["I1a"], "dispatch.rs", "const SWEEPS: usize = 2;", "const SWEEPS: usize = 2 ;"),
    ("2 space inside a continued string's value", ["I3"], "dispatch/tests.rs", "\\x20 expose n\\n\\", "\\x20  expose n\\n\\"),
    ("3 token change", ["I1b", "I2"], "dispatch/tests.rs", "ObjRef::NIL", "ObjRef::NUL"),
    ("4 tuple loses its trailing comma", ["I1b", "I2"], "dispatch/tests.rs",
     "String::from_utf8_lossy(&outcome.stderr).into_owned(),\n    )",
     "String::from_utf8_lossy(&outcome.stderr).into_owned()\n    )"),
    ("5 tuple gains a trailing comma", ["I1b", "I2"], "dispatch/tests.rs", "(93, 903));", "(93, 903,));"),
]
missed = 0
for name, expect, rel, old, new in CONTROLS:
    d = os.path.join(work, name.split()[0])
    shutil.rmtree(d, ignore_errors=True)
    os.makedirs(d)
    shutil.copy(os.path.join(post, "dispatch.rs"), d)
    shutil.copytree(os.path.join(post, "dispatch"), os.path.join(d, "dispatch"))
    p = os.path.join(d, rel); s = open(p).read()
    assert s.count(old) >= 1, (rel, old)
    i = s.find(old); open(p, "w").write(s[:i] + new + s[i + len(old):])
    r = subprocess.run([sys.executable, os.path.join(here, "instruments.py"), pre, d, removed, os.path.join(d, "out"), "--tests"], capture_output=True, text=True)
    seen = [tag for tag in expect if any(line.startswith(tag + " ") for line in r.stdout.splitlines())]
    ok = r.returncode == 1 and seen == expect
    missed += not ok
    print(f"control {name}: exit {r.returncode}, expected {expect}, reported {seen}: {'CAUGHT' if ok else 'MISSED'}")
    print("  " + (r.stdout + r.stderr).strip().replace("\n", "\n  "))
print(f"{len(CONTROLS) - missed} of {len(CONTROLS)} controls caught as expected")
sys.exit(1 if missed else 0)
