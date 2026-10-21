c1 = .routines~r~copy
c2 = .routines~i~copy
c3 = .routines~r~copy
.routines['Y'] = c1
call cy
.routines['Y'] = c2
call cy
.routines['Y'] = c3
call cy
exit 0
cy:
  signal on syntax name s
  call y 93900
  say 'answered' result
  return
s: do f over condition('O')~stackFrames
    e = f~executable
    if f~type == 'ROUTINE' then say 'raised' f~name (e == c1) (e == c2) (e == c3)
  end
  return
::routine i external 'LIBRARY orxfunction TestIntArg'
::routine r external 'LIBRARY orxfunction TestRaiseException0'
