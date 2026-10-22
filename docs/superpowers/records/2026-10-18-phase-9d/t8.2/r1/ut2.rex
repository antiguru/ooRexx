call on user u name h
call t
say 'back' result
say 'interpreted' ir()
exit
h: c = condition('O'); say 'h' c~hasIndex('RESULT') c~result; return
::routine t
signal on user u
call r
return 'np'
u: raise propagate
r: raise user u return 'w1'
::routine ir
call on user u name oh
interpret "raise user u return 'from-interpret'"
return 'fell through'
oh: c = condition('O'); say 'oh' c~hasIndex('RESULT') c~result; return
