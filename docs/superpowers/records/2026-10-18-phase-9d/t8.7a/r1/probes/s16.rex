trace i
v = .array~new
o = .k~new
w = .message~new(o, "R", "I", >v)~send
say w~class
exit
::class k
::method r
  use strict arg >x
  return >x
::method "[]"
  use strict arg >x
  return >x
::method "+"
  use strict arg x
  return >x
::routine f
  use strict arg >x
  return >x
::routine g
  use strict arg x
  return .array~of(x, x)
