# A notifier that raises in a started activity

Found by Phase 6 S2-S5 Task 7. Not fixed.

When a started message completes and an object its `~notify` named raises from
`messageComplete`, the oracle runs the notifier three times and reports the
condition twice; this crate runs it once and reports it once, as the started
activity's untrapped failure (`Interp::run_started`). The synchronous case
(`m~send`, the condition trapped by the sender) agrees.

```rexx
m = .message~new('abc','length')
m~notify(.bad~new)
m~start
m~wait
say 'main' m~result
call SysSleep 0.2
say 'end'
::class bad inherit MessageNotification
::method messageComplete
  say 'in notifier'
  return 1/0
```

Oracle stdout `in notifier`, `main 3`, `in notifier`, `in notifier`, `end`, rc 0,
stderr the 42.3 report twice. This crate: `in notifier`, `main 3`, `end`, rc 0,
the report once.
