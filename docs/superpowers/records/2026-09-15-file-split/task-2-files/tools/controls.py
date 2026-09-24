"""Negative controls for instruments 1-3: copy commit 1's POST tree, plant
three defects a pure move must not contain, and show each is reported.

  1. an unmoved line gains a space   (`const SWEEPS: usize = 2 ;`)      -> I1a
  2. a moved test's continued string gains a space inside its value
     (`\\x20 expose n` -> `\\x20  expose n`), token-blind by construction  -> I3
  3. a moved test's token changes    (first `ObjRef::NIL` -> `ObjRef::NUL`) -> I1b, I2

usage: controls.py PRE_ROOT POST_ROOT REMOVED_JSON WORKDIR
"""
import os, shutil, subprocess, sys
pre, post, removed, work = sys.argv[1:5]
shutil.rmtree(work, ignore_errors=True)
os.makedirs(work)
shutil.copy(os.path.join(post, "dispatch.rs"), work)
shutil.copytree(os.path.join(post, "dispatch"), os.path.join(work, "dispatch"))
def edit(rel, old, new):
    p = os.path.join(work, rel); s = open(p).read()
    assert s.count(old) >= 1, (rel, old)
    i = s.find(old); open(p, "w").write(s[:i] + new + s[i + len(old):])
edit("dispatch.rs", "const SWEEPS: usize = 2;", "const SWEEPS: usize = 2 ;")
edit("dispatch/tests.rs", "\\x20 expose n\\n\\", "\\x20  expose n\\n\\")
edit("dispatch/tests.rs", "ObjRef::NIL", "ObjRef::NUL")
here = os.path.dirname(os.path.abspath(__file__))
r = subprocess.run([sys.executable, os.path.join(here, "instruments.py"), pre, work, removed, os.path.join(work, "out"), "--tests"], capture_output=True, text=True)
print(f"instruments.py exit {r.returncode} (1 expected)")
print(r.stdout)
