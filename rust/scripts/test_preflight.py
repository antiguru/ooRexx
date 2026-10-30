#!/usr/bin/env python3
"""Check checkout-local inputs before running Rust workspace gates."""

import os
from pathlib import Path
import sys


NATIVE = tuple(
    f"build/lib/lib{name}.so"
    for name in ("rexx", "rexxapi", "rxregexp", "rxmath", "orxmethod", "orxfunction", "rxsock")
)
SUITE_FILES = (
    "ootest/ooTest.frm", "ootest/framework/OOREXXUNIT.CLS",
    "ootest/testOORexx.rex", "ootest/worker.rex",
)
SUITE_DIRS = tuple(f"ootest/ooRexx/{name}" for name in (
    "base/expressions", "base/keyword", "base/bif", "API",
))
DOCS = tuple(f"oodocs/rexxref/en-US/{name}.xml" for name in (
    "provide", "fundclasses", "collclasses", "utilityclasses", "streamclasses",
))


def missing_inputs(root):
    """Return missing files and directories relative to this checkout."""
    return [path for path in (*NATIVE, *SUITE_FILES, *DOCS)
            if not (root / path).is_file()] + [
                path for path in SUITE_DIRS if not (root / path).is_dir()]


def main(argv):
    root = Path(__file__).resolve().parents[2]
    missing = missing_inputs(root)
    if missing:
        print(f"Rust test prerequisites missing in {root}:", file=sys.stderr)
        for path in missing:
            print(f"  {root / path}", file=sys.stderr)
        print("Run from the checkout root:\n"
              "  memcap 8G cmake -S . -B build -DCMAKE_POLICY_VERSION_MINIMUM=3.5 -DCMAKE_BUILD_TYPE=Release\n"
              "  memcap 8G cmake --build build --target rxregexp rxmath orxmethod orxfunction rxsock -j4\n"
              "  svn checkout -r 13178 https://svn.code.sf.net/p/oorexx/code-0/test/trunk ootest\n"
              "  svn checkout -r 13198 https://svn.code.sf.net/p/oorexx/code-0/docs/trunk/rexxref oodocs/rexxref\n"
              "Keep these checkout-local libraries separate from the read-only oracle build.\n"
              "Then rerun: python3 rust/scripts/test_preflight.py", file=sys.stderr)
        return 1
    if argv:
        if argv[0] != "--" or len(argv) == 1:
            print("usage: test_preflight.py [-- COMMAND ARG ...]", file=sys.stderr)
            return 2
        os.execvp(argv[1], argv[1:])
    print(f"Rust test prerequisites present in {root}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
