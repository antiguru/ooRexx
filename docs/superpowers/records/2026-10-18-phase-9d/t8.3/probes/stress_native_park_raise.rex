c1 = .routines~r~copy
c2 = .routines~i~copy
.routines['Y'] = c1
call cy 'raised first'
.routines['Y'] = c2
call cy 'answered then'
exit 0
cy: procedure expose c1 c2
  use arg tag
  signal on syntax name s
  call y 93900
  say tag 'answered' result
  return
s: say tag 'raised' condition('O')~code
  return
::routine i external 'LIBRARY orxfunction TestIntArg'
::routine r external 'LIBRARY orxfunction TestRaiseException0'
