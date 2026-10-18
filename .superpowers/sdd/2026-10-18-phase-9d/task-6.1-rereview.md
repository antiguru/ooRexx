# Task 6.1 fix round 2: re-review

Reviewed `e27a7f843..ade6fd223` (head checked out is `38c2c81f2`, which adds records only). Release build of the crate, oracle, and 9c pin run from fresh directories.

## Verdict: approved, no open findings

## Findings

1. Dot-led last component after a directory (extension rule): fixed. `program_extension` scans from the end for `/` or `.` and skips only the first char, which is `extractExtension` (`SysFileSystem.cpp:256`) and `hasExtension` (`:322`). A multibyte first char is equivalent, since the oracle's skipped byte 0 is the lead byte and the continuation bytes are never `/` or `.`.
2. `/x` has no directory: fixed. `program_directory` skips the first char, which is `extractDirectory` (`:224`). The oracle's loop is `while (pathName < endPtr)`, so `/` at index 0 and `.` at index 0 are never examined. The one-char names `.` and `/` give `None` on both sides.
3. Current directory before the parent package's directory: witnessed by `rqd_cwd`. The oracle, crate and pin all answer `current directory` on the oracle's side and the crate matches (the pin does not, it fails earlier on the whole file).

## Checks asked for

* Replaced test `a_leading_dot_is_not_an_extension` asserted the per-component rule, which the oracle code contradicts (`.hid` after `/` is examined at index >= 1). Confirmed wrong. The replacement asserts the oracle's rule.
* `requires_dynamic_search_start.rex` run unchanged on oracle and crate: byte-identical, including `syntax 43.901 Could not find file "rqd/.hid" for ::REQUIRES.` The pin differs (as expected, it fails earlier).
* Plain-program regression: 25 names (`a.cls`, `a`, `x.rex`, `x`, `.x.rex`, `.x`, `m.b.c`, `m.b.c.rex`, `m.b`, `d.ir/f`, `d.ir/f.rex` with a dotted directory, `sub/.s`, `sub/.s.rex`, `./sub/.s`, `./.x.rex`, `bare`, `bare.rex`, `dd/n`, `dd/n.x`, `./a`, `a.`, `nonexist`, `/nonexist`, `.`, `..`), each by external `CALL` and `::REQUIRES`, with files for most. Crate equals oracle on every line (stdout, stderr, rc). Pin differs from the oracle only on `sub/.s` and `sub/.s.rex` (loads `sub/.s.rex` where the oracle loads `sub/.s`), which is the fixed defect.

## Out-of-scope observations

None.
