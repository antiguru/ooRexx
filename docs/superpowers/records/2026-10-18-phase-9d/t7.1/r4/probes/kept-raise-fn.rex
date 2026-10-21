/* kept-raise.rex in function-call form and through INTERPRET-free CALL VALUE-less forms */
.routines['Y'] = .routines~i~copy
call fy 1
.routines['Y'] = .routines~r~copy
call fy 'abc'
call fy 2
exit 0
fy: procedure
  use arg v
  signal on syntax name s
  z = y(v)
  say v 'answered' z
  return
s: say v 'raised' condition('O')~code
  return
::routine i external 'LIBRARY orxfunction TestIntArg'
::routine r external 'LIBRARY orxfunction TestRaiseException0'
