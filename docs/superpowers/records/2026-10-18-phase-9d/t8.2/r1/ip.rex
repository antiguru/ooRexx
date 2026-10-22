call on user u name h
x = 'z'
interpret "call r"
say 'x' result
call q
exit
h: c = condition('O'); say 'h' c~hasIndex('RESULT') c~result; return
r: raise user u return 'in-interpret'
q: interpret "raise user u exit 'interp-exit'"
