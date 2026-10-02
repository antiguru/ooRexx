# REPLY in an UNINIT run at termination never runs its continuation

Found by the Task 6 review (M2), pre-existing (the deferral before Task 6 behaved the same). An
`UNINIT` that replies while the interpreter terminates leaves its continuation filed and unrun,
with no report: `execute_on` runs `terminate()` after `run_started_to_end`, and nothing runs the
activities the termination's `UNINIT`s spawn.

```rexx
o = .k~new
say 'main end'
::class k
::method uninit
  say 'uninit before reply'
  reply
  say 'uninit after reply'
```

Oracle: `main end`, `uninit before reply`, `uninit after reply` (3/3 in Task 6 fix round 1; the
review measured the third line 2 of 3, a race with process exit). This crate: the first two lines
only, rc 0.

Phase 6 S2-S5 Task 9 fix round 1 (ruling P39): the program end does not wait for an activity a
termination `UNINIT` starts, so this continuation never runs here. On the oracle it is a race with
process exit whose outcome depends on load: the Task 9 review measured the third line in 7 of 30
runs at load average 29 and 30 of 30 at load 14.
