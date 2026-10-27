q = .array~of('p','q'); d = .directory~new; o = .k~new
signal on nostring name ns1
say (>q)
say 'o' (>o) length(>o)
say 'qq' ((>q) || 'x')~changestr('0a'x, ',')
signal on nostring name ns2
say 'd' (>d)
exit
ns1: say 'NOSTRING 1' condition('d'); exit
ns2: say 'NOSTRING 2' condition('d') condition('o')~additional~class~id; exit
::class k
::method makestring; return 'kmake'
