# EXIT in a CALL ON handler run at the REPLY clause's end

Found by the Task 6 re-review (O2), present before Task 6's fixes (939fdace1 behaves the same).
A `CALL ON` handler that the REPLY clause's end runs, and that ends with `EXIT`, ends the method
before its split on the oracle, which then reports that the method returned no result to the
sender. This crate hands the sender the reply value. A REPLY followed by a plain `EXIT`
matches (review probe `hexit2`). Probe: scratchpad `rr-t6/probes/hexit.rex`.

```rexx
.local~main = .context~thread
say 'got' .k~new~m
call SysSleep 0.2
say 'main end'
::class k
::method m
  call on notready name nr
  reply 'v' || linein('/nonexistent/file/x')
  say 'rest on main' (.context~thread = .local~main)
  return
nr:
  say 'handler on main' (.context~thread = .local~main)
  exit
```

Oracle (7/7 in the review), rc 165:

```
handler on main 1
```
stderr:
```
     2 *-* say 'got' .k~new~m
Error 91 running .../p.rex line 2:  No result object.
Error 91.999:  Message "M" did not return a result.
```

This crate (Task 6 fix round 2, release), rc 0, stderr empty:

```
handler on main 1
got v
main end
```
