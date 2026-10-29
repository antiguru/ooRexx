v = .array~new
call h >v
call g >v
exit
h: use arg >x
  r = >x
  say r~name
  trace i
  r = >x
  trace off
  return
g: procedure
  use arg >y
  trace i
  r = >y
  w = (>y)~name
  trace off
  say w
  return
