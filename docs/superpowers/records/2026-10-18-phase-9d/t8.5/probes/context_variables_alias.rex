say q.
call r >q.
a = 1
call s >a
exit
r: procedure
  use arg >z.
  v = .context~variables
  say 'stem' v~hasIndex('Z.') v~hasIndex('Q.') v~items
  return
s: procedure
  use arg >z
  v = .context~variables
  say 'simple' v~hasIndex('Z') v~hasIndex('A') v~items
  return
