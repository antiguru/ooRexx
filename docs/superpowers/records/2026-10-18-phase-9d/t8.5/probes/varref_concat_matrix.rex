e = .array~new(2); q = .array~of('p','q'); o = .k~new; n = .nostr~new; s = 'abc'
say 'L1' e || 'x'
say 'L2' (e || 'x')
say 'L3' (q || 'x')
say 'L4' (o || 'x')
say 'L5' (n || 'x')
say 'R1' ((>e) || 'x')
say 'R2' ((>q) || 'x')
say 'R3' ((>o) || 'x')
say 'R4' ((>n) || 'x')
say 'R5' ((>s) || 'x')
say 'R6' ((>e) 'x')
say 'R7' ((>e)'x')
say 'R8' ((>q) 'x')
say 'R9' ((>o)'x')
say 'S1' ('x' || (>e))
say 'S2' ('x' || (>o))
say 'M1' (>e)~'||'('x')
say 'M2' e~'||'('x')
say 'M3' o~'||'('x')
::class k
::method string; return 'kstr'
::method makestring; return 'kmake'
::class nostr
::method string; return 'nstr'
