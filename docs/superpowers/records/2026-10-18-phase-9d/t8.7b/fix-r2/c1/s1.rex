call on user sig name hs
call rs 'one'
say 'not reached'
exit
rs: parse arg dd; raise user sig description (dd) additional (dd'-add') return
hs:
  say 'hs' condition('d') condition('a')
  call rs 'two'
  call rs 'three'
  signal on user sig name sg
  say 'hs not reached'
  return
sg:
  co = condition('o')
  say 'sg' condition('d') condition('a') condition('i') sigl co~description co~additional~class co~position co~condition
  signal on user sig name sg2
  call rs 'four'
  exit
sg2:
  say 'sg2' condition('d')
  exit
