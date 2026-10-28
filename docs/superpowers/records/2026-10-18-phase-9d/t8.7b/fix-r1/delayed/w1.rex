-- direct RAISE ... RETURN inside the handler
call on user ping name h
n = 0
call rp 'first'
say 'after first, n =' n
-- SIGNAL ON re-issued in the handler
call on user sig name hs
call rs 'one'
say 'not reached'
exit
rp: parse arg dd; raise user ping description (dd) return
rs: parse arg dd; raise user sig description (dd) return
h:
  n = n + 1
  say 'h' n condition('d')
  if n = 1 then raise user ping description 'again' return
  say 'h' n 'returns'
  return
hs:
  say 'hs' condition('d') condition('i')
  call rs 'two'
  say 'hs raised again'
  signal on user sig name sg
  say 'hs not reached'
  return
sg:
  say 'sg' condition('d') condition('i') sigl
  say 'sg end'
  exit
