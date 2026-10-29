call on user sig name hs
call rs 'one'
exit
rs: parse arg dd; raise user sig description (dd) additional (dd'-add') return (dd'-res')
hs:
  call rs 'two'
  signal on user sig name sg
  return
sg:
  co = condition('o')
  say 'sg' condition('d') condition('a') co~result condition('i')
  exit
