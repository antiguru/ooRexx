"""Transfer a waitable child through exec, with a destructor-poll control."""

import os
import subprocess
import sys


def waitable_child():
    child = subprocess.Popen([sys.executable, "-c", "raise SystemExit(5)"])
    # Observe exit without consuming status. No scheduling delay is assumed.
    status = os.waitid(os.P_PID, child.pid, os.WEXITED | os.WNOWAIT)
    assert status.si_code == os.CLD_EXITED and status.si_status == 5, status
    return child


if sys.argv[1] == "retained":
    # Successful exec preserves this reference without running __del__.
    child = waitable_child()
elif sys.argv[1] == "dropped":
    child = waitable_child()
    del child
else:
    raise ValueError(sys.argv[1])
os.execv(sys.argv[2], sys.argv[2:])
