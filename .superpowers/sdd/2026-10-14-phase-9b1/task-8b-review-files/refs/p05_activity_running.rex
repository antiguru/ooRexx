y = copies('rr', 23)
w = .w~new
m = w~start('spin', >y)
drop y
call filler
say 'p05 result' m~result
r = mk()
m2 = w~start('spin', r)
drop r
call filler
say 'p05b result' m2~result
exit
mk: procedure
  v = copies('vv', 11)
  return >v
::class w
::method spin
  use arg ref
  call filler
  return length(ref~value) left(ref~value, 3)
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
