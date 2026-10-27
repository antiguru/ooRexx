n = .nstr~new; d = .directory~new; k = .kmake~new; q = .array~of('p','q')
say 'n1' ('nstr' = (>n)) ('a NSTR' = (>n)) ('nstr' = n)
say 'd1' ('a Directory' = (>d))
say 'k1' ('kmake' = (>k)) ('kmake' = k) ('kmake' == (>k))
say 'q1' ('p' || '0a'x || 'q' = (>q))
say 'k2' ('kmake' < (>k)) ('kmakf' > (>k)) ('kmake' \= (>k))
x = 'kmake'; say 'k3' (x == (>k))
say 'k4' 'kmake'~equals(>k)  'kmake'~compareTo(>k)
say 'k5' 'kmake'~caselessEquals(>k)
say 'k6' ('ab' || 'c')~pos(>k) 'kmakeX'~pos(>k)
say 'k7' 'ab'~left(1) || (>k)
::class kmake
::method makestring; return 'kmake'
::class nstr
::method string; return 'nstr'
