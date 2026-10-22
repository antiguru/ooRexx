call on failure name h
zr = ra() + ra()
say "zr" zr
exit
ra: raise failure 6 return 1
h: say "handler" condition("C") condition("D"); return
