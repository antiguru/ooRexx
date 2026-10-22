/* A raise inside a Rexx ::routine's own activation leaves its call site
   keeping nothing, so the next call looks the name up again; a routine that
   returned, even with nothing for a function call to use, stays kept. In the
   function and the CALL form. */
.routines['Y'] = .routines~bad
do i = 1 to 2
  call site i
end
.routines['Y'] = .routines~bad
do i = 3 to 4
  call csite i
end
.routines['Y'] = .routines~nodata
do i = 5 to 6
  call nsite i
end
.routines['Y'] = .routines~nodata
do i = 7 to 8
  call ncsite i
end
exit
site: procedure
  use arg i
  signal on syntax name h
  z = y()
  say i 'answered' z
  return
h: say i 'raised' condition('O')~code
  .routines['Y'] = .routines~good
  return
csite: procedure
  use arg i
  signal on syntax name h2
  call y
  say i 'call answered' result
  .routines['Y'] = .routines~good
  return
h2: say i 'call raised' condition('O')~code
  .routines['Y'] = .routines~good
  return
nsite: procedure
  use arg i
  signal on syntax name h3
  z = y()
  say i 'answered' z
  return
h3: say i 'raised' condition('O')~code
  .routines['Y'] = .routines~good
  return
ncsite: procedure
  use arg i
  signal on syntax name h4
  call y
  say i 'call answered' result
  .routines['Y'] = .routines~good
  return
h4: say i 'call raised' condition('O')~code
  .routines['Y'] = .routines~good
  return
::routine bad; return 1 + 'x'
::routine good; return 'good'
::routine nodata; return
