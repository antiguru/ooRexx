r = mk()
call filler
say 'q16a' length(r~value) left(r~value,3)
x = copies('vb', 11)
call v >x
call filler
say 'q16b' length(x) left(x, 3)
exit
mk: procedure
  y = copies('vy', 12)
  r = >y
  call value 'Y', copies('vz', 13)
  vars = .context~variables
  return r
v: procedure
  use arg >z
  call value 'Z', copies('vc', 15)
  call filler
  d = .context~variables
  say 'q16v' length(d['Z'])
  return
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
