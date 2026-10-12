# A Stream object's writes are visible by name before it is closed

Found by the surface Task 6 review (review-s6, its M7), not made by Task 6: the
parent build `362fd648c` answers the same as HEAD. Queued, not fixed. No ledger
entry mentions it.

## Probe

`streamflush.rex`, no handler and no extension:

```rexx
/* a Stream object target read by name before the object is closed: shell, then no command at all */
sh = .stream~new('sh.txt')
address system 'echo hi' with output using (sh)
say rc lines('sh.txt')
s2 = .stream~new('s2.txt')
s2~lineout('direct')
say lines('s2.txt')
```

## Outputs

Re-run 2026-09-27 by the Task 6 fix round, each side from a fresh `mktemp -d`,
the oracle under `( ulimit -v 1048576; LD_LIBRARY_PATH=<oracle>/build/lib
<oracle>/build/bin/rexx streamflush.rex )`, the crate as release `rexx-run`
built at `b06b278ce` plus the fix round's working tree
(`scratchpad/fix-s6/cmp.sh`), three descriptors separately.

Oracle, rc 0, stderr empty:

```
0 0
0
```

This crate, rc 0, stderr empty:

```
0 1
1
```

While the object is open, `LINES` by name reports no line on the oracle and
the written line here. Why is not established: nothing here was read from the
C++ stream code. The difference is what made the Stream-object line of the
review's `streams.rex` differ.
