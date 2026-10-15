o = .k~new
r = o~go
call filler
say 'q18a' length(r~value) left(r~value, 3)
o~release = 1
call filler
say 'q18b' length(r~value) left(r~value, 3)
exit
::class k
::attribute release
::method go
  expose release
  release = 0
  y = copies('rp', 14)
  reply >y
  guard on when release = 1
  y = copies('rq', 15)
  return
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
