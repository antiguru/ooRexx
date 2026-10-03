# A concatenation in a non-first argument refuses "a compiled call op does not name a call of its own body"

Found by Phase 6 S2-S5 Task 12 (TEST_BASE_ALARM's `self~assertTrue(1, 'a' d)` with `d` a
`.DateTime`); minimal repro by the Task 12 review (m4). Pre-existing: same at 217f33a2c, single
activity, not concurrency. It needs the concatenation of an object with a STRING method in a
non-first argument of a call, function or send.

```rexx
call m 1, 'a' .s~new
say 'ok'
exit
m: return
::class s
::method string
  return 'S'
```

Oracle: `ok`, rc 0. This crate, base and head: refuses with "a compiled call op does not name a
call of its own body".
