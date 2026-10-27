say 'a' symbol('A.1') symbol('A.')
say 'b' p. symbol('P.')
x = 'c' symbol('B.') b.7; say x symbol('B.')
call value 'C.1'; say 'd' symbol('C.')
call value 'D.1', 5; say 'e' symbol('D.')
say 'f' var('E.1') symbol('E.')
interpret 'say f.1'; say 'g' symbol('F.')
parse value 'q' with g.1 .; say 'h' symbol('G.') g.1
i = 1; say 'i' h.i symbol('H.')
call t1; say 'j' symbol('J.') symbol('K.')
say 'k' k.1~length symbol('K.')
say 'l' (l.1 == 'L.1') symbol('L.')
drop z.1 z.2
say 'm' symbol('Z.')
v = .context~variables
say 'n' v~hasIndex('A.') v~hasIndex('P.') v~hasIndex('B.') v~hasIndex('Z.') v~hasIndex('H.')
exit
t1: procedure expose j.
  say j.1
  return
