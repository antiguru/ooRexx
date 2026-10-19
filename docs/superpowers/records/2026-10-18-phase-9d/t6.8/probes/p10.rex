c = 3
signal on novalue name n1
call pf
say 'after pf' c symbol('nolist2') symbol('e')
signal on syntax name s1
call pg
say 'after pg' c symbol('nolist3')
exit
n1: say 'n1' condition('d') 'c='c symbol('c')
c = 'h'; nolist2 = 'x'; e = 'e'
return
s1: say 's1' condition('o')~code 'c='c
c = 'h2'; nolist3 = 'y'
return
pf: procedure expose c (nolist2) e
say 'pf body'
return
pg: procedure expose c (nolist3)
say 'pg body'
return
::options novalue syntax
