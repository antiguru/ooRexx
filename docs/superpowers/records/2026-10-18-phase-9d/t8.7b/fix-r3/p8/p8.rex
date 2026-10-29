call on user sig name hs
call rs 'one'
say 'after one'
call on user c name hc
call rc 'one'
say 'after c'
call on user s2 name hs2
call r2 'one'
exit
rs: parse arg dd; raise user sig description (dd) return
rc: parse arg dd; raise user c description (dd) return
r2: parse arg dd; raise user s2 description (dd) return
hs:
  say 'hs' condition('d') condition('o')~position sigl
  if condition('d') = 'one' then call rs 'two'
  return
hc:
  say 'hc' condition('d') condition('o')~position sigl
  if condition('d') = 'one' then do
    call rc 'two'
    call on user c name hc
  end
  return
hs2:
  say 'hs2' condition('d') condition('o')~position sigl
  call r2 'two'
  signal on user s2 name sg
  return
sg:
  say 'sg' condition('d') condition('o')~position sigl condition('o')~traceback~firstitem
  exit
