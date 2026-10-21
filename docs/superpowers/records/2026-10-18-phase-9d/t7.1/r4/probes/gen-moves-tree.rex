/* a site kept a lookup-every-call answer (an external file); this call's
   argument rebinds the name, so the call resolves afresh after its
   arguments to a .ROUTINES copy, and raises through it. The oracle keeps a
   routine only once a call returns, so the next call sees the next rebind. */
ci = .routines~i~copy
cr = .routines~r~copy
call fy 1
call fy 'swap'
.routines['Y'] = ci
call fy 3
.routines~remove('Y')
call cy 1
call cy 'swap'
.routines['Y'] = ci
call cy 3
exit 0
fy: procedure expose ci cr
  use arg v
  signal on syntax name s
  z = y(f(v)~~length)
  say 'fn' v 'answered' z
  return
s: say 'fn' v 'raised' condition('O')~code
  return
cy: procedure expose ci cr
  use arg v
  signal on syntax name s2
  call y f(v)~~length
  say 'call' v 'answered' result
  return
s2: say 'call' v 'raised' condition('O')~code
  return
f: procedure expose ci cr
  if arg(1) <> 'swap' then return arg(1)
  .routines['Y'] = cr
  return 2
::routine i external 'LIBRARY orxfunction TestIntArg'
::routine r external 'LIBRARY orxfunction TestRaiseException0'
