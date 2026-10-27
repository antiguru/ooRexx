call p1; say 'p1' symbol('A.')
call p2; say 'p2' symbol('B.') symbol('B.1')
call p3; say 'p3' symbol('C.')
list = 'D.1'; drop (list); say 'd' symbol('D.')
o = .k~new; o~m; o~n
e.x = 1; drop e.; say 'e' symbol('E.') symbol('E.X')
call p4 f.; say 'f' symbol('F.')
r = g.1; call p5; say 'g' symbol('G.')
interpret 'drop h.1'; say 'h' symbol('H.')
say 'i' symbol('I.1') symbol('I.')
v = .context~variables; say 'v' v~hasIndex('I.') v~hasIndex('A.') v~hasIndex('C.')
exit
p1: procedure expose a.
  drop a.1
  return
p2: procedure expose b.1
  say 'in p2' symbol('B.') symbol('B.1')
  drop b.1
  return
p3: procedure expose c.
  say 'p3 in' symbol('C.') symbol('C.1')
  return
p4: procedure
  use arg z.
  say 'p4' symbol('Z.')
  return
p5: procedure
  return
::class k
::method m
  expose s.
  drop s.1
::method n
  expose s.
  say 'n' symbol('S.') s.~items
