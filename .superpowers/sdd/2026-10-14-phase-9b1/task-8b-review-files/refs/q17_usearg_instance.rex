o = .k~new
r = o~ref
call q r
drop o
call filler
say 'q17' length(r~value) left(r~value, 3)
exit
q: procedure
  use arg >z
  z = copies('q7', 9)
  call filler
  return
::class k
::method ref
  expose v
  v = copies('ov', 13)
  return >v
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
