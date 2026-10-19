list = 'c'; c = 3; i = 2
call pg
signal on novalue name n1
call pf
say 'after pf' c
exit
n1: say 'n1' condition('d') symbol('c') symbol('list') symbol('i') symbol('nolist2')
c = 'set in handler'
return
pg: procedure expose c list
say 'pg' symbol('c') c
return
pf: procedure expose c list (nolist2) i
say 'pf body'
return
