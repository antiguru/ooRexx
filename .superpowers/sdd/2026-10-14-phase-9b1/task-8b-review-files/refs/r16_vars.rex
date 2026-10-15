x = copies('vb', 11)
call v >x
say 'r16b' length(x) left(x, 3)
exit
v: procedure
  use arg >z
  call value 'Z', copies('vc', 15)
  d = .context~variables
  say 'r16v' d['Z']~class~id length(d['Z']) d['Z']
  return
