# The `<I<` line of a method run by `Message~triggered` is not routed

Found by the Phase 6 S2-S5 Task 9 review (z6); base a4bde5677 and Task 9 answer the same. With
`.TRACEOUTPUT` routed to a sink, the `<I<` line of a method `Message~triggered` ran reaches stdout
as plain text instead of the sink; every Alarm or Ticker target traced to a sink meets it.

```rexx
s = .sink~new
zz = .traceoutput~destination(s)
.message~new(.w~new, 'two')~triggered(.nil)
zz = .traceoutput~destination(.stderr)
do ln over s~seen
  say 'sink:' ln
end
exit
::class w
::method two
  trace a
  return
::class sink
::method init
  expose seen
  seen = .array~new
::method seen
  expose seen
  return seen
::method lineout unguarded
  expose seen
  use arg v
  seen~append(subword(v~traceline, 1, 3))
  return 0
```

Oracle stdout: `sink: >I> Method "TWO"`, `sink: 12 *-* return`, `sink: <I< Method "TWO"`, rc 0.
This crate: the `<I< Method "TWO" ...` line first, unrouted, then the two sink lines, rc 0.
