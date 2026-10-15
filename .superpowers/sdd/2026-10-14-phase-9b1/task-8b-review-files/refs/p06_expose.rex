y = copies('ee', 12)
r = >y
call p
call filler
say 'p06a' length(r~value) left(r~value,3) length(y) left(y,3)
r2 = q()
call filler
say 'p06b' length(r2~value) left(r2~value,3) length(z) left(z,3)
drop z
call filler
say 'p06c' symbol('z') r2~value~length
exit
p: procedure expose y
  y = copies('pp', 14)
  return
q: procedure expose z
  z = copies('zz', 16)
  return >z
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
