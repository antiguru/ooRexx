/* An attribute's Method object keeps its scope through setMethod, and its
 * pair answers through the .nil scope's pool. */
m = .u~method('A')
say m~scope~id
o = .t~new
say o~go(m)
say m~scope~id
::class t
::method go
  use arg m
  self~setMethod('a', m)
  self~setMethod('a=', .u~method('A='))
  self~a = 9
  return self~a self~run(.u~method('A'))
::class u
::attribute a
