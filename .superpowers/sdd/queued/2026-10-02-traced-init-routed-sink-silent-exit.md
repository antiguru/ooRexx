# A traced INIT with .traceOutput routed to a Rexx sink ends the program silently

Present at a4bde5677 (before Phase 6 Task 9). With `.traceOutput` routed to a Rexx object whose
`lineout` stores the line, a class whose `INIT` runs `trace a` ends the whole program at `.w~new`
with rc 165 and no error report; main's later lines never run. The oracle exits rc 0 and prints
the sink's lines. Found by the Task 9 re-review (scratch `rev-t9/u/i3.rex`):

```rexx
s = .sink~new
zz = .traceoutput~destination(s)
o = .w~new
zz = .traceoutput~destination(.stderr)
say 'after' s~seen~items
do ln over s~seen
  say ln
end
::class w
::method init
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

The existing `.superpowers/sdd/queued/2026-10-01-routine-call-context-name.md` also shows in
TraceObject STACKFRAME NAME since Task 9 (`.routines~rr~call` names `CALL`, oracle `RR`).
