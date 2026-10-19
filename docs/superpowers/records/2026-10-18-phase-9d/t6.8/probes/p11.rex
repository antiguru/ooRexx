c = 3; sel = '9bad'; d = 4
signal on syntax name s1
call pg
say 'after pg' c d
signal on syntax name s1
call ph
say 'after ph' c d
exit
s1: say 's1' condition('o')~code 'c='c 'd='d 'sel='sel
c = 'h2'; sel = 'z'; d = 'h3'
return
pg: procedure expose c d (sel)
say 'pg body'
return
ph: procedure expose c d
nop; x = 1 + 'a'
return
