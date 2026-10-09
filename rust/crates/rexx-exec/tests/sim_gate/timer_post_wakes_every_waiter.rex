/* One post to a timer wakes every activity waiting on it (the seeded gate's
   witness for a post waking only the last waiter). A line starting FAIL is
   the gate's assertion. The natives are bound to a class of this program, as
   CoreClasses.orx binds them. */
o = .t~new
r = o~tickerCreate
h = o~handle
m1 = .message~new(o, 'tickerWait', 'I', h, 0, 20000); m1~start
m2 = .message~new(o, 'tickerWait', 'I', h, 0, 20000); m2~start
call SysSleep 0.2
r = o~tickerStop(h)
call SysSleep 0.2
if m1~completed & m2~completed then say 'one post woke both'
else say 'FAIL one post woke' m1~completed m2~completed
exit
::class t
::method alarmStart unguarded external 'LIBRARY REXX alarm_startTimer'
::method alarmStop unguarded external 'LIBRARY REXX alarm_stopTimer'
::method tickerCreate unguarded external 'LIBRARY REXX ticker_createTimer'
::method tickerWait unguarded external 'LIBRARY REXX ticker_waitTimer'
::method tickerStop unguarded external 'LIBRARY REXX ticker_stopTimer'
::method handle unguarded
  expose eventSemHandle
  return eventSemHandle
::method started unguarded
  expose timerStarted
  return timerStarted
::method cancelNow unguarded
  expose canceled
  canceled = .true
