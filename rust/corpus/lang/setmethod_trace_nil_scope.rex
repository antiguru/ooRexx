/* A ::METHOD body run through setMethod announces the .nil scope. */
say .t~new~go
::class t
::method go
  self~setMethod('hm', .u~method('M'))
  return self~hm
::class u
::method m
  trace r
  x = 1
  return x
