list = 'c'; c = 3; i = 2; a.2 = 'two'
trace i
call pe
trace off
say 'back' symbol('nolist2')
call on any
signal on novalue name n1
call pf
say 'after pf'
exit
n1: say 'n1' condition('d') symbol('c') symbol('list') symbol('i')
return
pe: procedure expose a.i (list) a.1
return
pf: procedure expose c list (nolist2) i
say 'pf body'
return
