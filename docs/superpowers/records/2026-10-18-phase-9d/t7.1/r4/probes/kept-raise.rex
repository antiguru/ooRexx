/* a site that kept a copy after a return, then raised through that copy:
   the oracle keeps the copy at the instruction, so a rebind is not seen */
c2 = .routines~i~copy
c3 = .routines~r~copy
.routines['Y'] = c2
call cy 1
.routines['Y'] = c3
call cy 'abc'
call cy 2
.routines['W'] = .routines~i~copy
call cw 1
.routines['W'] = .routines~r~copy
call cw 'arg raises'
call cw 3
exit 0
cy: procedure expose c2 c3
  use arg v
  signal on syntax name s
  call y v
  say v 'answered' result
  return
s: say v 'raised' condition('O')~code
  return
cw: procedure
  use arg v
  signal on syntax name s1
  call w f(v)
  say v 'answered' result
  return
s1: say v 'raised' condition('O')~code
  return
f: procedure
  if arg(1) = 'arg raises' then return 1 + 'x'
  return arg(1)
::routine i external 'LIBRARY orxfunction TestIntArg'
::routine r external 'LIBRARY orxfunction TestRaiseException0'
