/* the oracle reads a site's kept routine after the arguments run: an
   argument that re-enters the same site (which returns, so it keeps i) and
   then rebinds the name sends the outer call to the kept i */
ci = .routines~i~copy
cr = .routines~r~copy
.routines['Y'] = ci
call fy 'x', 0
.routines['Y'] = ci
call cy 'x', 0
exit 0
fy: procedure expose ci cr
  use arg v, d
  signal on syntax name s
  z = y(g(v, d, 'f')~~length)
  say 'fn' d v 'answered' z
  return
s: say 'fn' d v 'raised' condition('O')~code
  return
cy: procedure expose ci cr
  use arg v, d
  signal on syntax name s2
  call y g(v, d, 'c')~~length
  say 'call' d v 'answered' result
  return
s2: say 'call' d v 'raised' condition('O')~code
  return
g: procedure expose ci cr
  use arg v, d, form
  if d > 0 then return v
  if form = 'f' then call fy 5, 1
  else call cy 5, 1
  .routines['Y'] = cr
  return 2
::routine i external 'LIBRARY orxfunction TestIntArg'
::routine r external 'LIBRARY orxfunction TestRaiseException0'
