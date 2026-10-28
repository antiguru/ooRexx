.local~setMethod('UNKNOWN', 'raise syntax 98.900 array("from unknown")')
o = .object~new
o~setMethod('f', .method~new('f', 'say "in f"; use arg n; return n'))
say 'zork:' fcc('ZORK')
say 'after'
::routine fcc external "LIBRARY orxfunction TestFindContextClass"
