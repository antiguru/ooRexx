r = mk()
call filler
say 'p09a' length(r~value) left(r~value,3)
x = copies('ix', 10)
interpret 'call q >x'
call filler
say 'p09b' length(x) left(x,3)
interpret 'y = copies(''iy'', 8); r2 = >y'
drop y
call filler
say 'p09c' symbol('y') (r2~value == 'Y')
exit
mk: procedure
  interpret 'y = copies(''in'', 12); r = >y'
  return r
q: procedure
  use arg >z
  call filler
  z = copies('qz', 11)
  return
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
