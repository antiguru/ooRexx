# A translation error inside INTERPRET omits the interpreted clause's traceback line

Found by the Phase 6 S2-S5 Task 12 review (m6). Pre-existing: same at 217f33a2c; not related to
parking. The oracle's error report includes a traceback line for the interpreted clause; this
crate's omits it. Compare stderr on the oracle and on ours.

```rexx
/* GUARD WHEN inside INTERPRET */
o = .k~new
m = o~start('waiter')
call SysSleep 0.1
o~bump
say m~result
::class k
::method init
  expose v
  v = 0
::method bump
  expose v
  v += 1
::method waiter
  expose v
  interpret 'guard on when v > 0'
  return 'woke' v
```
