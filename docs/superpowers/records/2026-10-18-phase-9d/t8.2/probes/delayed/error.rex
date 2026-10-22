call on error name h
zr = ra() + ra()
say "zr" zr
exit
ra: raise error 5 return 1
h: say "handler" condition("C") condition("D"); return
