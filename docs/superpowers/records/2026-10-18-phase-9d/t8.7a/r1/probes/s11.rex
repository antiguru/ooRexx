trace i
v = .array~new
w = g(v)
say w~items
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
