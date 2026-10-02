# A REPLY's continuation keeps the replier's guard (Task 11)

The oracle's continuation takes over the replying method's guard lock, so a guarded send the
sender makes after the REPLY waits until the rest of the method releases it and reads what it
wrote. This crate's continuation holds no guard: since ruling P34 the sender runs on after a REPLY
and reads the value from before the write. Task 6 fix round 1 moved the witness below out of the
corpus (it was `rust/corpus/lang/reply_seen_by_a_guarded_send.rex`, passing only because a REPLY
yielded to its continuation, R-T6-4). It returns to the corpus with Task 11's guard transfer.

```rexx
/* What the rest of a replied method writes is what a later guarded send of
   the sender reads: the oracle's continuation holds the object's guard. */
o = .R~new
say 'v' f(o~m('argument-value-long'))
say 'after' o~seen
call sub o~m('second-argument-long')
say 'res' result o~seen
exit
sub:
  return 'sub('arg(1)')'
::routine f
  use arg v
  return 'f('v')'
::class R
::method init
  expose seen
  seen = 'none'
::method m
  expose seen
  use arg v
  reply 'replied('v')'
  seen = v
::method seen
  expose seen
  return seen
```

Oracle (30/30, Task 6 report), rc 0, stderr empty:

```
v f(replied(argument-value-long))
after argument-value-long
res sub(replied(second-argument-long)) second-argument-long
```

This crate after fix round 1, unswitched and `REXX_SWITCH_MODE=every`, rc 0, stderr empty:

```
v f(replied(argument-value-long))
after none
res sub(replied(second-argument-long)) none
```

A second shape, where the continuation parks before writing (Task 6 review M3, `init.rex`): the
oracle prints `made K second` 3/3, this crate `made K first`.

```rexx
q = .queue~new
o = .k~new(q)
say 'made' o~class~id o~v
do until q~items >= 1
  call SysSleep 0.01
end
say 'rest' q~pull o~v
::class k
::method init
  expose v
  use arg q
  v = 'first'
  reply
  call SysSleep 0.02
  v = 'second'
  q~queue('init rest ran')
::method v
  expose v
  return v
```
