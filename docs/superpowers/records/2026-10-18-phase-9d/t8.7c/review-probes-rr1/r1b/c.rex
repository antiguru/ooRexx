kk = .kmake~new
nn = .nstr~new
k. = kk
v. = nn
a.KM = 'km-tail'
signal on nostring name h
call r1 >kk
call r2 >nn
say 'stem k' a.[k.]
say 'stem k2' a.[k., 1]
say 'stem v' a.[v.]
say 'end'
exit
r1: use arg >x; say 'ref k' a.[x]; return
r2: use arg >x; say 'ref v' a.[x]; return
h: say 'NOSTRING' condition('d') sigl; signal resume
resume:
exit
::class nstr
::method string; return 'NSTR'
::class kmake
::method makestring; return 'KM'
::method string; return 'KS'
