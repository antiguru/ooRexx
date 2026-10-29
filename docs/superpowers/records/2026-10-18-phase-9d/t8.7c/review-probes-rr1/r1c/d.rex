kk = .kmake~new
nn = .nstr~new
k. = kk
v. = nn
a.KM = 'km-tail'
say 'plain k' a.[kk]
say 'stem k' a.[k.]
say 'stem v untrapped' a.[v.]
say 'say v untrapped' v.
say 'concat v untrapped' v.'|'
x = v.
say 'x untrapped' x
say 'length v' length(v.)
signal on nostring name h
say 'plain v trapped' a.[nn]
exit
h: say 'NOSTRING' condition('d') sigl
exit
::class nstr
::method string; return 'NSTR'
::class kmake
::method makestring; return 'KM'
::method string; return 'KS'
