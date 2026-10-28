trace i
v = .array~new
o = .k~new
if o~r(>v)~class = .VariableReference then say "y"
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
