/* first call returns (site keeps i), rebind to r, raise twice through kept, then return */
.routines['Y'] = .routines~i~copy
call cy 1
.routines['Y'] = .routines~r~copy
call cy 'abc'
call cy 'def'
call cy 2
.routines['Z'] = .routines~i
call fz 1
.routines['Z'] = .routines~r
call fz 'abc'
call fz 'def'
call fz 2
exit 0
cy: procedure
  use arg v
  signal on syntax name s
  call y v
  say 'call' v 'answered' result
  return
s: say 'call' v 'raised' condition('O')~code
  return
fz: procedure
  use arg v
  signal on syntax name s2
  q = z(v)
  say 'fn' v 'answered' q
  return
s2: say 'fn' v 'raised' condition('O')~code
  return
::routine i external 'LIBRARY orxfunction TestIntArg'
::routine r external 'LIBRARY orxfunction TestRaiseException0'
