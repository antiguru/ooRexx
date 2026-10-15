o = .k~new
r = o~go
call filler
say 'r18a' length(r~value) left(r~value, 3)
o~release = 1
o~waitdone
say 'r18b' length(r~value) left(r~value, 3) o~seen
exit
::class k
::attribute release
::attribute seen
::method waitdone
  expose done
  guard on when done = 1
::method go
  expose release done seen
  release = 0
  done = 0
  y = copies('rp', 14)
  reply >y
  guard on when release = 1
  y = copies('rq', 15)
  seen = length(y)
  done = 1
  return
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
