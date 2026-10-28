/* A traced reference is held while its trace lines are written. */
y = 'why'
trace i
q = >y
r = <y
call show >y
trace off
say q r~value
do i = 1 to 2
  trace i
  s = >y
  trace off
end
say s~value
v = .array~new
o = .k~new
trace i
w = o~r(>v)
say o~r(>v)~class
interpret 'u = o[>v]'
v = 1
say o~r(>v)~class
trace off
say w~class u~class
exit

show:
  use arg >z
  say z
  return

::class k
::method r
  use strict arg >x
  return >x
::method '[]'
  use strict arg >x
  return >x
