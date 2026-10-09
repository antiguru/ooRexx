/* A pinned sleep that fails (fail=wait:1 in the seeded gate's row), then a
   park on a started activity's result: the failed sleep's deadline must not
   ready the parked main. The gate judges this program by the scheduler's
   invariants only: its stdout is not read. */
call time 'R'
call sleeper
m = .w~new~start('slow')
say 'result' m~result 'after' (time('E') >= 2.5)
exit
sleeper: procedure
  signal on syntax name caught
  interpret 'call SysSleep 1'
  say 'slept'
  return
caught:
  say 'caught' condition('O')~code
  return
::class w
::method slow
  call SysSleep 3
  return 'slow done'
