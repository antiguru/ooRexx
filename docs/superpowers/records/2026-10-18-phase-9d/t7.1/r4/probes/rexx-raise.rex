/* concern 2: a Rexx routine whose first call raised, then the name is rebound */
.routines['Y'] = .routines~ra
call cy 1
.routines['Y'] = .routines~rb
call cy 2
exit 0
cy: procedure
  use arg v
  signal on syntax name s
  call y v
  say v 'answered' result
  return
s: say v 'raised' condition('O')~code
  return
::routine ra
  raise syntax 93.900 array('ra')
::routine rb
  return 'rb'
