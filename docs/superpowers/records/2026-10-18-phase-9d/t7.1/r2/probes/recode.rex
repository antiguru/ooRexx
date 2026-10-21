/* rebind Y from a copy of one library routine to a copy of another */
.routines['Y'] = .routines~i~copy
call cy
.routines['Y'] = .routines~r~copy
call cy
.routines['Y'] = .routines~i
call cy
.routines['Y'] = .routines~r
call cy
.context~package~addRoutine('X', .routines~i~copy)
call cx
.context~package~addRoutine('X', .routines~r~copy)
call cx
exit 0
cy:
  signal on syntax name s
  r = y(93900)
  say 'answered' r
  return
s: say 'raised' condition('O')~code
  return
cx:
  signal on syntax name s2
  r = x(93900)
  say 'answered' r
  return
s2: say 'raised' condition('O')~code
  return
::routine i external 'LIBRARY orxfunction TestIntArg'
::routine r external 'LIBRARY orxfunction TestRaiseException0'
