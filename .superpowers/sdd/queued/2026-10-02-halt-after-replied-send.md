# Message~halt after a ~send whose method replied

Found by the Task 6 re-review (O3), present before Task 6's fixes (939fdace1 behaves the same).
A `Message~send` whose method replies leaves the message's activity as the sender's on the
oracle, so `m~halt` answers `1` and halts the sending activity (main): 4.1 at the next clause
boundary. This crate answers `0` and runs on. Probe: scratchpad `rr-t6/probes/pend4.rex`.

```rexx
/* halt the message's activity before the continuation has run */
o = .k~new
m = .message~new(o, 'm')
say m~send
say m~halt
call SysSleep 0.3
say 'main end'
::class k
::method m
  reply 'v'
  do i = 1 to 3; say 'rest' i; end
```

Oracle (3/3 in the review), rc 252:

```
v
1
rest 1
rest 2
rest 3
```
stderr:
```
     5 *-* say m~halt
Error 4 running .../p.rex line 5:  Program interrupted.
Error 4.1:  Program interrupted with HALT condition.
```

This crate (Task 6 fix round 2, release), rc 0, stderr empty:

```
v
0
rest 1
rest 2
rest 3
main end
```
