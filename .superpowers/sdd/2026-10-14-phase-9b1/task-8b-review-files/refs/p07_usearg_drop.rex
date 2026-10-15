x = copies('ua', 10)
call q >x
call filler
say 'p07a' symbol('x') length(x) left(x, 3)
w = copies('wa', 10)
r = >w
drop r
call filler
say 'p07b' length(w) left(w,3)
call t >w
call filler
say 'p07c' symbol('w')
exit
q: procedure
  use arg >z
  drop z
  call filler
  z = copies('ub', 12)
  call filler
  return
t: procedure
  use arg >z
  drop z
  call filler
  return
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
