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
