/* An argument that calls through its own call site, which returns and so
   keeps the routine, then rebinds the name and raises: the outer call's
   raise leaves what the inner call kept. With the site holding nothing on
   entry (n) and after a raising call (e), in the function and the CALL
   form. */
ci = .routines~i~copy
cr = .routines~r~copy
call fy 1, 1
call fy 'x', 0
call fy 3, 1
.routines~remove('Y')
call cy 1, 1
call cy 'x', 0
call cy 3, 1
.routines~remove('Y')
call fn 'x', 0
call fn 3, 1
.routines~remove('Y')
call cn 'x', 0
call cn 3, 1
exit 0
fy: procedure expose ci cr
  use arg v, d
  signal on syntax name s1
  z = y(g(v, d, 'fy')~~length)
  say 'fn-e' d v 'answered' z
  return
s1: say 'fn-e' d v 'raised' condition('O')~code
  return
cy: procedure expose ci cr
  use arg v, d
  signal on syntax name s2
  call y g(v, d, 'cy')~~length
  say 'call-e' d v 'answered' result
  return
s2: say 'call-e' d v 'raised' condition('O')~code
  return
fn: procedure expose ci cr
  use arg v, d
  signal on syntax name s3
  z = y(g(v, d, 'fn')~~length)
  say 'fn-n' d v 'answered' z
  return
s3: say 'fn-n' d v 'raised' condition('O')~code
  return
cn: procedure expose ci cr
  use arg v, d
  signal on syntax name s4
  call y g(v, d, 'cn')~~length
  say 'call-n' d v 'answered' result
  return
s4: say 'call-n' d v 'raised' condition('O')~code
  return
g: procedure expose ci cr
  use arg v, d, form
  if d > 0 then return v
  .routines['Y'] = ci
  interpret 'call' form '5, 1'
  .routines['Y'] = cr
  return 1 + 'x'
::routine i external 'LIBRARY orxfunction TestIntArg'
::routine r external 'LIBRARY orxfunction TestRaiseException0'
