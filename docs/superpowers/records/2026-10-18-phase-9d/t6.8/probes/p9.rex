c = 3; d = 4
signal on novalue name n1
call pf
say 'after pf' c d
call pg
say 'after pg' c d
call ph
say 'after ph' c d
exit
n1: say 'n1' condition('d') 'c='c 'd='d
c = 'h'; d = 'h'
return
pf: procedure expose c (nolist2)
say 'pf body'
return
pg: procedure expose (nolist3) d
say 'pg body'
return
ph: procedure expose c
c = 'ph'
nop; x = undefinedvar
say 'ph body'
return
