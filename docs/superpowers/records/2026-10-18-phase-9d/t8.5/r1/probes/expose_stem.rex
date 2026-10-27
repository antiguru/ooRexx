call p3; say 'after p3' symbol('C.')
call p4; say 'after p4' symbol('D.')
o = .k~new; o~m; o~n
exit
p3: procedure expose c.
  say 'p3 in' symbol('C.')
  return
p4: procedure expose (lst) d.
  return
::class k
::method m
  expose s.
  say 'm' symbol('S.')
::method n
  expose s.
  say 'n' symbol('S.')
